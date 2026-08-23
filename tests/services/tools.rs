use loco_rs::{prelude::Error, testing::prelude::*};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serial_test::serial;
use todo::{
    app::App,
    models::{_entities::boards as boards_entity, boards, projects, todos, users, users_projects},
    services::tools::{execute, execute_with_context, validate_context, ToolContext},
};

async fn create_project(
    ctx: &loco_rs::app::AppContext,
    owner: &users::Model,
    title: &str,
) -> projects::Model {
    let project = projects::ActiveModel {
        title: Set(title.to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();

    users_projects::ActiveModel {
        user_id: Set(owner.id),
        project_id: Set(project.id),
        role: Set("owner".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();

    project
}

async fn first_board(ctx: &loco_rs::app::AppContext, project: &projects::Model) -> boards::Model {
    boards::Entity::find()
        .filter(boards_entity::Column::ProjectId.eq(project.id))
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
#[serial]
async fn shared_tools_apply_context_and_enforce_project_boundaries() {
    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();
    let ctx = &boot.app_context;

    let owner = users::Entity::find_by_id(1)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let outsider = users::Entity::find_by_id(2)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();

    let project = create_project(ctx, &owner, "Private project").await;
    let board = first_board(ctx, &project).await;
    let todo = todos::Model::create(
        &ctx.db,
        &todos::TodoParams {
            title: "Protected todo".to_owned(),
            details: None,
            tags: None,
            locked: None,
            parent_pid: None,
        },
        &board,
        &owner,
    )
    .await
    .unwrap();

    let selected = ToolContext {
        project_pid: Some(project.pid.to_string()),
        board_pid: Some(board.pid.to_string()),
        todo_pid: Some(todo.pid.to_string()),
    };
    execute_with_context(ctx, &owner, "get_todo", &serde_json::json!({}), &selected)
        .await
        .unwrap();

    let (expanded, selected_project) = validate_context(
        ctx,
        &owner,
        &ToolContext {
            todo_pid: Some(todo.pid.to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(expanded.board_pid, Some(board.pid.to_string()));
    assert_eq!(expanded.project_pid, Some(project.pid.to_string()));
    assert_eq!(selected_project.unwrap().id, project.id);

    let result = validate_context(
        ctx,
        &outsider,
        &ToolContext {
            todo_pid: Some(todo.pid.to_string()),
            ..Default::default()
        },
    )
    .await;
    assert!(matches!(result, Err(Error::Unauthorized(_))));

    let result = execute(
        ctx,
        &outsider,
        "get_todo",
        &serde_json::json!({ "todo_pid": todo.pid.to_string() }),
    )
    .await;
    assert!(matches!(result, Err(Error::Unauthorized(_))));

    users_projects::ActiveModel {
        user_id: Set(outsider.id),
        project_id: Set(project.id),
        role: Set("member".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap();

    execute(
        ctx,
        &outsider,
        "get_todo",
        &serde_json::json!({ "todo_pid": todo.pid.to_string() }),
    )
    .await
    .unwrap();

    let result = execute(
        ctx,
        &outsider,
        "update_project",
        &serde_json::json!({
            "project_pid": project.pid.to_string(),
            "title": "Unauthorized rename"
        }),
    )
    .await;
    assert!(matches!(result, Err(Error::Unauthorized(_))));

    let unchanged = projects::Entity::find_by_id(project.id)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.title, "Private project");

    let other_project = create_project(ctx, &owner, "Other project").await;
    let other_board = first_board(ctx, &other_project).await;
    let result = execute(
        ctx,
        &owner,
        "update_todo",
        &serde_json::json!({
            "todo_pid": todo.pid.to_string(),
            "board_pid": other_board.pid.to_string()
        }),
    )
    .await;
    assert!(matches!(result, Err(Error::BadRequest(_))));

    let unchanged = todos::Entity::find_by_id(todo.id)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.board_id, board.id);
}

#[tokio::test]
#[serial]
async fn child_todo_must_use_its_parent_board() {
    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();
    let ctx = &boot.app_context;
    let owner = users::Entity::find_by_id(1)
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    let project = create_project(ctx, &owner, "Parent rules").await;
    let mut project_boards = boards::Entity::find()
        .filter(boards_entity::Column::ProjectId.eq(project.id))
        .all(&ctx.db)
        .await
        .unwrap();
    let first = project_boards.remove(0);
    let second = project_boards.remove(0);
    let parent = todos::Model::create(
        &ctx.db,
        &todos::TodoParams {
            title: "Parent".to_owned(),
            details: None,
            tags: None,
            locked: None,
            parent_pid: None,
        },
        &first,
        &owner,
    )
    .await
    .unwrap();

    let result = execute(
        ctx,
        &owner,
        "add_todo",
        &serde_json::json!({
            "title": "Invalid child",
            "board_pid": second.pid.to_string(),
            "parent_pid": parent.pid.to_string()
        }),
    )
    .await;
    assert!(matches!(result, Err(Error::BadRequest(_))));
}
