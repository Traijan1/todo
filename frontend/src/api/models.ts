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

export interface Tag {
  pid: string;
  title: string;
  color?: string;
}

export interface Todo {
  pid: string;
  title: string;
  details?: string;
  board_pid: string;
  position: number;
  created_at: string;
  updated_at: string;
  tags?: Tag[];
}

export interface Board {
  pid: string;
  title: string;
  position: number;
  created_at: string;
  updated_at: string;
  todos: Todo[];
}
