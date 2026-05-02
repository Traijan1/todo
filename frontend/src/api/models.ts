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

export interface Board {
  pid: string;
  project_id: number; // Internal ID for relations, though usually we fetch by project PID
  title: string;
  created_at: string;
  updated_at: string;
}

export interface Todo {
  id: number;
  pid: string;
  title: string;
  content?: string;
  done: boolean;
  created_at: string;
  updated_at: string;
}
