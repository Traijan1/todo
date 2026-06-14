use serde_json::{json, Value};

pub fn get_tools_list() -> Value {
    json!([
        // ── Projects ──────────────────────────────────────────────────────
        {
            "name": "get_projects",
            "description": "List all todo projects",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "add_project",
            "description": "Create a new project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Project title" },
                    "description": { "type": "string", "description": "Project description (optional)" }
                },
                "required": ["title"]
            }
        },
        {
            "name": "update_project",
            "description": "Update a project's title or description",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": { "type": "string", "description": "UUID (PID) of the project" },
                    "title": { "type": "string", "description": "New title" },
                    "description": { "type": "string", "description": "New description" }
                },
                "required": ["project_pid"]
            }
        },
        {
            "name": "delete_project",
            "description": "Delete a project permanently",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": { "type": "string", "description": "UUID (PID) of the project" }
                },
                "required": ["project_pid"]
            }
        },
        // ── Boards ────────────────────────────────────────────────────────
        {
            "name": "get_boards",
            "description": "List all boards of a project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": { "type": "string", "description": "UUID (PID) of the project" }
                },
                "required": ["project_pid"]
            }
        },
        {
            "name": "add_board",
            "description": "Create a board inside a project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": { "type": "string", "description": "UUID (PID) of the project" },
                    "title": { "type": "string", "description": "Board title" }
                },
                "required": ["project_pid", "title"]
            }
        },
        {
            "name": "update_board",
            "description": "Rename a board",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "board_pid": { "type": "string", "description": "UUID (PID) of the board" },
                    "title": { "type": "string", "description": "New title" }
                },
                "required": ["board_pid", "title"]
            }
        },
        {
            "name": "delete_board",
            "description": "Delete a board and all its todos",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "board_pid": { "type": "string", "description": "UUID (PID) of the board" }
                },
                "required": ["board_pid"]
            }
        },
        // ── Todos ─────────────────────────────────────────────────────────
        {
            "name": "get_todos",
            "description": "List all todos of a project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": { "type": "string", "description": "UUID (PID) of the project" }
                },
                "required": ["project_pid"]
            }
        },
        {
            "name": "get_todo",
            "description": "Get details of a single todo",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": { "type": "string", "description": "UUID (PID) of the todo" }
                },
                "required": ["todo_pid"]
            }
        },
        {
            "name": "add_todo",
            "description": "Create a todo in a board",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "board_pid": { "type": "string", "description": "UUID (PID) of the board" },
                    "title": { "type": "string", "description": "Todo title" },
                    "details": { "type": "string", "description": "Details in HTML (optional)" },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tag PIDs to assign (optional)"
                    }
                },
                "required": ["board_pid", "title"]
            }
        },
        {
            "name": "update_todo",
            "description": "Update a todo's title, details, board or tags",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": { "type": "string", "description": "UUID (PID) of the todo" },
                    "title": { "type": "string", "description": "New title" },
                    "details": { "type": "string", "description": "New details in HTML" },
                    "board_pid": { "type": "string", "description": "Move to this board (PID)" },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Full list of tag PIDs to set (replaces existing tags)"
                    }
                },
                "required": ["todo_pid"]
            }
        },
        {
            "name": "delete_todo",
            "description": "Delete a todo permanently",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": { "type": "string", "description": "UUID (PID) of the todo" }
                },
                "required": ["todo_pid"]
            }
        },
        // ── Tags ──────────────────────────────────────────────────────────
        {
            "name": "get_tags",
            "description": "List all available tags",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "create_tag",
            "description": "Create a new tag with optional color",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Tag name" },
                    "color": { "type": "string", "description": "Hex color e.g. #E0BBE4 (optional)" }
                },
                "required": ["title"]
            }
        },
        {
            "name": "update_tag",
            "description": "Rename a tag or change its color",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tag_pid": { "type": "string", "description": "UUID (PID) of the tag" },
                    "title": { "type": "string", "description": "New name" },
                    "color": { "type": "string", "description": "New hex color" }
                },
                "required": ["tag_pid"]
            }
        },
        {
            "name": "delete_tag",
            "description": "Delete a tag (removes it from all todos)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tag_pid": { "type": "string", "description": "UUID (PID) of the tag" }
                },
                "required": ["tag_pid"]
            }
        },
        {
            "name": "add_tag_to_todo",
            "description": "Add an existing tag to a todo",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": { "type": "string", "description": "UUID (PID) of the todo" },
                    "tag_pid": { "type": "string", "description": "UUID (PID) of the tag" }
                },
                "required": ["todo_pid", "tag_pid"]
            }
        },
        {
            "name": "remove_tag_from_todo",
            "description": "Remove a tag from a todo",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": { "type": "string", "description": "UUID (PID) of the todo" },
                    "tag_pid": { "type": "string", "description": "UUID (PID) of the tag" }
                },
                "required": ["todo_pid", "tag_pid"]
            }
        }
    ])
}
