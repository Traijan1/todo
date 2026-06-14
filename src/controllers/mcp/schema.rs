use serde_json::{json, Value};

pub fn get_tools_list() -> Value {
    json!([
        {
            "name": "get_projects",
            "description": "List all todo projects",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "get_todos",
            "description": "List all todos of a project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the project"
                    }
                }
            }
        },
        {
            "name": "get_todo",
            "description": "Retrieve information about a specific todo",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the todo"
                    }
                }
            }
        },
        {
            "name": "get_boards",
            "description": "List all boards of a project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the project"
                    }
                }
            }
        },
        {
            "name": "add_todo",
            "description": "Create a todo for a project board",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "board_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the board"
                    },
                    "title": {
                        "type": "string",
                        "description": "The title of the todo"
                    },
                    "details": {
                        "type": "string",
                        "description": "The details of the todo (optional) in HTML"
                    }
                },
                "required": ["board_pid", "title"]
            }
        },
        {
            "name": "add_board",
            "description": "Create a board for a project",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the project"
                    },
                    "title": {
                        "type": "string",
                        "description": "The title of the board"
                    },
                }
            }
        },
        {
            "name": "get_tags",
            "description": "List all available tags",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "add_tag_to_todo",
            "description": "Add an existing tag to a todo",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "todo_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the todo"
                    },
                    "tag_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the tag"
                    }
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
                    "todo_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the todo"
                    },
                    "tag_pid": {
                        "type": "string",
                        "description": "The UUID (PID) of the tag"
                    }
                },
                "required": ["todo_pid", "tag_pid"]
            }
        },
    ])
}
