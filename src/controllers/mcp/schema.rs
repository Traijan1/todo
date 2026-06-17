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
        {
            "name": "reorder_boards",
            "description": "Set the display order of boards within a project. Pass all board PIDs in the desired order.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "board_pids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "All board PIDs of the project in the desired order"
                    }
                },
                "required": ["board_pids"]
            }
        },
        // ── Todos ─────────────────────────────────────────────────────────
        {
            "name": "get_todos",
            "description": "List top-level todos of a project (subtasks are not included; use get_todo to see a todo's subtasks). Optionally filtered by board, tags, and/or full-text search. Always includes tag data per todo.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": { "type": "string", "description": "UUID (PID) of the project" },
                    "board_pid": { "type": "string", "description": "Filter to a specific board (optional)" },
                    "tag_pids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter to todos that have at least one of these tag PIDs (optional)"
                    },
                    "search": { "type": "string", "description": "Full-text search on todo title (optional)" }
                },
                "required": ["project_pid"]
            }
        },
        {
            "name": "get_todo",
            "description": "Get details of a single todo, including its tags, subtasks, and comments. The `details` field is the first truth — the authoritative description of the task. Comments (in the `comments` array) extend or update that truth chronologically.",
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
            "description": "Create a todo or subtask. Provide board_pid for a top-level todo, or parent_pid to create a subtask (board is inherited from parent).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "board_pid": { "type": "string", "description": "UUID (PID) of the board. Required unless parent_pid is provided." },
                    "parent_pid": { "type": "string", "description": "UUID (PID) of the parent todo. Creates a subtask; board is inherited from the parent." },
                    "title": { "type": "string", "description": "Todo title" },
                    "details": { "type": "string", "description": "Details in HTML (optional)" },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tag names to assign, e.g. [\"urgent\", \"bug\"]. Tags are created automatically if they don't exist."
                    }
                },
                "required": ["title"]
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
                        "description": "Full list of tag names to set, e.g. [\"urgent\", \"bug\"]. Replaces existing tags. Tags are created automatically if they don't exist."
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
        {
            "name": "reorder_todos",
            "description": "Set the display order of todos within a board. Pass all top-level todo PIDs of that board in the desired order.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "All top-level todo PIDs of the board in the desired order"
                    }
                },
                "required": ["todo_pids"]
            }
        },
        // ── Tags ──────────────────────────────────────────────────────────
        {
            "name": "get_tags",
            "description": "List all tags for a project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": { "type": "string", "description": "UUID (PID) of the project" }
                },
                "required": ["project_pid"]
            }
        },
        {
            "name": "create_tag",
            "description": "Create a new tag for a project with optional color",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": { "type": "string", "description": "UUID (PID) of the project" },
                    "title": { "type": "string", "description": "Tag name" },
                    "color": { "type": "string", "description": "Hex color e.g. #E0BBE4 (optional)" }
                },
                "required": ["project_pid", "title"]
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
        },
        // ── Comments ──────────────────────────────────────────────────────
        {
            "name": "get_comments",
            "description": "Get all comments for a todo, ordered chronologically. The todo's `details` field is the source of truth (first truth) — comments extend or update it over time.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": { "type": "string", "description": "UUID (PID) of the todo" }
                },
                "required": ["todo_pid"]
            }
        },
        {
            "name": "add_comment",
            "description": "Add a comment to a todo. The comment is marked as AI-authored (is_ai=true). Use this to extend or update the todo's current state — the `details` field is the first truth, comments build on top of it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": { "type": "string", "description": "UUID (PID) of the todo" },
                    "content": { "type": "string", "description": "Comment text" }
                },
                "required": ["todo_pid", "content"]
            }
        }
    ])
}
