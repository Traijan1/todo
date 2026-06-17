export interface User {
  pid: string;
  name: string;
  email: string;
  is_verified: boolean;
}

export interface Project {
  pid: string;
  title: string;
  description?: string;
  mcp_expose_comments: boolean;
  created_at: string;
  updated_at: string;
}

export interface Tag {
  pid: string;
  title: string;
  color?: string;
}

export interface SubtaskItem {
  pid: string;
  title: string;
  locked: boolean;
  tags?: Tag[];
}

export interface Todo {
  pid: string;
  title: string;
  details?: string;
  board_pid: string;
  position: number;
  locked: boolean;
  parent_pid?: string;
  subtasks?: SubtaskItem[];
  created_at: string;
  updated_at: string;
  tags?: Tag[];
}

export interface Comment {
  pid: string;
  author: string;
  content: string;
  is_ai: boolean;
  created_at: string;
}

export interface Board {
  pid: string;
  title: string;
  position: number;
  created_at: string;
  updated_at: string;
  todos: Todo[];
}
