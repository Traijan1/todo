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
  created_at: string;
  updated_at: string;
}

export interface Todo {
  id: number;
  pid: string;
  title: string;
  details?: string;
  board_id: number;
  created_at: string;
  updated_at: string;
}

export interface Board {
  pid: string;
  title: string;
  created_at: string;
  updated_at: string;
  todos: Todo[];
}
