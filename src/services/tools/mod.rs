mod actions;
mod schema;

use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::models::{
    boards, projects, tags, todos, users,
    users_projects::{self, Column as MembershipColumn},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ToolContext {
    pub project_pid: Option<String>,
    pub board_pid: Option<String>,
    pub todo_pid: Option<String>,
}

pub fn definitions() -> Value {
    schema::get_tools_list()
}

/// Convert the canonical MCP tool schemas to Ollama's function-tool format.
pub fn ollama_definitions() -> Value {
    let tools = definitions()
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tool| {
            Some(json!({
                "type": "function",
                "function": {
                    "name": tool.get("name")?.clone(),
                    "description": tool.get("description").cloned().unwrap_or(Value::String(String::new())),
                    "parameters": tool.get("inputSchema").cloned().unwrap_or_else(|| json!({ "type": "object" })),
                }
            }))
        })
        .collect();
    Value::Array(tools)
}

pub fn is_known_tool(name: &str) -> bool {
    definitions().as_array().is_some_and(|tools| {
        tools
            .iter()
            .any(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
    })
}

pub async fn execute(
    ctx: &AppContext,
    user: &users::Model,
    name: &str,
    args: &Value,
) -> Result<Value> {
    if !is_known_tool(name) {
        return Err(Error::BadRequest(format!("Unknown tool: {name}")));
    }
    authorize(ctx, user, name, args).await?;
    dispatch(ctx, user, name, args).await
}

pub async fn execute_with_context(
    ctx: &AppContext,
    user: &users::Model,
    name: &str,
    args: &Value,
    tool_context: &ToolContext,
) -> Result<Value> {
    let args = apply_context(args, tool_context)?;
    execute(ctx, user, name, &args).await
}

fn apply_context(args: &Value, context: &ToolContext) -> Result<Value> {
    let mut args = match args {
        Value::Object(values) => values.clone(),
        Value::Null => Map::new(),
        _ => return Err(Error::BadRequest("Tool arguments must be an object".into())),
    };

    for (key, value) in [
        ("project_pid", context.project_pid.as_ref()),
        ("board_pid", context.board_pid.as_ref()),
        ("todo_pid", context.todo_pid.as_ref()),
    ] {
        if !args.contains_key(key) {
            if let Some(value) = value {
                args.insert(key.into(), Value::String(value.clone()));
            }
        }
    }

    Ok(Value::Object(args))
}

async fn dispatch(
    ctx: &AppContext,
    user: &users::Model,
    name: &str,
    args: &Value,
) -> Result<Value> {
    match name {
        "get_projects" => actions::get_projects(ctx, user).await,
        "add_project" => actions::add_project(ctx, args, user).await,
        "update_project" => actions::update_project(ctx, args).await,
        "delete_project" => actions::delete_project(ctx, args).await,
        "get_boards" => actions::get_boards(ctx, args).await,
        "add_board" => actions::add_board(ctx, args).await,
        "update_board" => actions::update_board(ctx, args).await,
        "delete_board" => actions::delete_board(ctx, args).await,
        "reorder_boards" => actions::reorder_boards(ctx, args).await,
        "get_todos" => actions::get_todos(ctx, args).await,
        "get_todo" => actions::get_todo(ctx, args).await,
        "add_todo" => actions::add_todo(ctx, args, user).await,
        "update_todo" => actions::update_todo(ctx, args, user).await,
        "delete_todo" => actions::delete_todo(ctx, args).await,
        "reorder_todos" => actions::reorder_todos(ctx, args).await,
        "get_tags" => actions::get_tags(ctx, args).await,
        "create_tag" => actions::create_tag(ctx, args).await,
        "update_tag" => actions::update_tag(ctx, args).await,
        "delete_tag" => actions::delete_tag(ctx, args).await,
        "add_tag_to_todo" => actions::add_tag_to_todo(ctx, args).await,
        "remove_tag_from_todo" => actions::remove_tag_from_todo(ctx, args).await,
        "get_comments" => actions::get_comments(ctx, args).await,
        "add_comment" => actions::add_comment(ctx, args, user).await,
        "start_timer" => actions::start_timer(ctx, args, user).await,
        "stop_timer" => actions::stop_timer(ctx, args, user).await,
        "get_time" => actions::get_time(ctx, args).await,
        _ => Err(Error::BadRequest(format!("Unknown tool: {name}"))),
    }
}

fn required_string<'a>(args: &'a Value, name: &str) -> Result<&'a str> {
    args.get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| Error::BadRequest(format!("{name} required")))
}

fn required_strings(args: &Value, name: &str) -> Result<Vec<String>> {
    let raw_values = args
        .get(name)
        .and_then(Value::as_array)
        .ok_or_else(|| Error::BadRequest(format!("{name} required")))?;
    if raw_values.is_empty() {
        return Err(Error::BadRequest(format!("{name} must not be empty")));
    }
    raw_values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .map(ToString::to_string)
                .ok_or_else(|| Error::BadRequest(format!("{name} must contain only strings")))
        })
        .collect()
}

async fn membership(
    ctx: &AppContext,
    user: &users::Model,
    project_id: i32,
) -> Result<users_projects::Model> {
    users_projects::Entity::find()
        .filter(MembershipColumn::UserId.eq(user.id))
        .filter(MembershipColumn::ProjectId.eq(project_id))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::Unauthorized("You do not have access to this project".into()))
}

async fn project_access(
    ctx: &AppContext,
    user: &users::Model,
    project_pid: &str,
    owner_only: bool,
) -> Result<projects::Model> {
    let project = projects::Model::find_by_pid(&ctx.db, project_pid).await?;
    let membership = membership(ctx, user, project.id).await?;
    if owner_only && membership.role != "owner" {
        return Err(Error::Unauthorized(
            "Only project owners may perform this action".into(),
        ));
    }
    Ok(project)
}

async fn board_access(
    ctx: &AppContext,
    user: &users::Model,
    board_pid: &str,
) -> Result<boards::Model> {
    let board = boards::Model::find_by_pid(&ctx.db, board_pid).await?;
    membership(ctx, user, board.project_id).await?;
    Ok(board)
}

async fn todo_access(
    ctx: &AppContext,
    user: &users::Model,
    todo_pid: &str,
) -> Result<(todos::Model, boards::Model)> {
    let todo = todos::Model::find_by_pid(&ctx.db, todo_pid).await?;
    if todo.locked {
        return Err(Error::NotFound);
    }
    let board = boards::Entity::find_by_id(todo.board_id)
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    membership(ctx, user, board.project_id).await?;
    Ok((todo, board))
}

async fn tag_access(ctx: &AppContext, user: &users::Model, tag_pid: &str) -> Result<tags::Model> {
    let tag = tags::Model::find_by_pid(&ctx.db, tag_pid).await?;
    membership(ctx, user, tag.project_id).await?;
    Ok(tag)
}

async fn authorize(ctx: &AppContext, user: &users::Model, name: &str, args: &Value) -> Result<()> {
    match name {
        "get_projects" | "add_project" => {}
        "update_project" | "delete_project" => {
            project_access(ctx, user, required_string(args, "project_pid")?, true).await?;
        }
        "get_boards" | "add_board" | "get_tags" | "create_tag" => {
            project_access(ctx, user, required_string(args, "project_pid")?, false).await?;
        }
        "get_todos" => {
            let project =
                project_access(ctx, user, required_string(args, "project_pid")?, false).await?;
            if let Some(board_pid) = args.get("board_pid").and_then(Value::as_str) {
                let board = board_access(ctx, user, board_pid).await?;
                if board.project_id != project.id {
                    return Err(Error::BadRequest(
                        "board_pid does not belong to project_pid".into(),
                    ));
                }
            }
            if let Some(tag_pids) = args.get("tag_pids").and_then(Value::as_array) {
                for tag_pid in tag_pids.iter().filter_map(Value::as_str) {
                    let tag = tag_access(ctx, user, tag_pid).await?;
                    if tag.project_id != project.id {
                        return Err(Error::BadRequest(
                            "tag_pids must belong to project_pid".into(),
                        ));
                    }
                }
            }
        }
        "update_board" | "delete_board" => {
            board_access(ctx, user, required_string(args, "board_pid")?).await?;
        }
        "reorder_boards" => {
            let board_pids = required_strings(args, "board_pids")?;
            let first = board_access(ctx, user, &board_pids[0]).await?;
            for board_pid in &board_pids[1..] {
                let board = board_access(ctx, user, board_pid).await?;
                if board.project_id != first.project_id {
                    return Err(Error::BadRequest(
                        "All boards must belong to the same project".into(),
                    ));
                }
            }
        }
        "get_todo" | "delete_todo" | "get_comments" | "add_comment" | "start_timer"
        | "stop_timer" | "get_time" => {
            todo_access(ctx, user, required_string(args, "todo_pid")?).await?;
        }
        "add_todo" => {
            let board = if let Some(board_pid) = args.get("board_pid").and_then(Value::as_str) {
                Some(board_access(ctx, user, board_pid).await?)
            } else {
                None
            };
            let parent = if let Some(parent_pid) = args.get("parent_pid").and_then(Value::as_str) {
                Some(todo_access(ctx, user, parent_pid).await?)
            } else {
                None
            };
            if board.is_none() && parent.is_none() {
                return Err(Error::BadRequest("board_pid or parent_pid required".into()));
            }
            if let (Some(board), Some((_, parent_board))) = (&board, &parent) {
                if board.id != parent_board.id {
                    return Err(Error::BadRequest(
                        "board_pid and parent_pid must belong to the same board".into(),
                    ));
                }
            }
        }
        "update_todo" => {
            let (_, source_board) =
                todo_access(ctx, user, required_string(args, "todo_pid")?).await?;
            if let Some(board_pid) = args.get("board_pid").and_then(Value::as_str) {
                let target_board = board_access(ctx, user, board_pid).await?;
                if source_board.project_id != target_board.project_id {
                    return Err(Error::BadRequest(
                        "Todos cannot be moved between projects".into(),
                    ));
                }
            }
        }
        "reorder_todos" => {
            let todo_pids = required_strings(args, "todo_pids")?;
            let (first, _) = todo_access(ctx, user, &todo_pids[0]).await?;
            for todo_pid in &todo_pids[1..] {
                let (todo, _) = todo_access(ctx, user, todo_pid).await?;
                if todo.board_id != first.board_id {
                    return Err(Error::BadRequest(
                        "All todos must belong to the same board".into(),
                    ));
                }
            }
        }
        "update_tag" | "delete_tag" => {
            tag_access(ctx, user, required_string(args, "tag_pid")?).await?;
        }
        "add_tag_to_todo" | "remove_tag_from_todo" => {
            let (_, board) = todo_access(ctx, user, required_string(args, "todo_pid")?).await?;
            let tag = tag_access(ctx, user, required_string(args, "tag_pid")?).await?;
            if board.project_id != tag.project_id {
                return Err(Error::BadRequest(
                    "Tag and todo must belong to the same project".into(),
                ));
            }
        }
        _ => return Err(Error::BadRequest(format!("Unknown tool: {name}"))),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_missing_context_without_overwriting_explicit_arguments() {
        let context = ToolContext {
            project_pid: Some("project-context".into()),
            board_pid: Some("board-context".into()),
            todo_pid: Some("todo-context".into()),
        };
        let args = json!({ "todo_pid": "todo-explicit", "title": "Renamed" });
        let merged = apply_context(&args, &context).unwrap();

        assert_eq!(merged["project_pid"], "project-context");
        assert_eq!(merged["board_pid"], "board-context");
        assert_eq!(merged["todo_pid"], "todo-explicit");
    }

    #[test]
    fn converts_all_mcp_definitions_for_ollama() {
        let mcp = definitions();
        let ollama = ollama_definitions();
        assert_eq!(
            mcp.as_array().unwrap().len(),
            ollama.as_array().unwrap().len()
        );
        assert!(ollama.as_array().unwrap().iter().all(|tool| {
            tool["type"] == "function"
                && tool["function"]["name"].is_string()
                && tool["function"]["parameters"].is_object()
        }));
    }
}
