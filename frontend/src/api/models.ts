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
  ai_provider?: string;
  ai_model?: string;
  ai_system_prompt?: string;
  created_at: string;
  updated_at: string;
}

export interface AiProvider {
  id: string;
  name: string;
  kind: string;
  default_model?: string;
}

export interface AiSettings {
  default_provider: string;
  providers: AiProvider[];
}

export interface AiModel {
  id: string;
  name: string;
  size?: number;
  details?: {
    family?: string;
    parameter_size?: string;
    quantization_level?: string;
  };
}

export interface AiTestResult {
  ok: boolean;
  provider_id: string;
  model: string;
  response: string;
  thinking?: string;
  duration_ms: number;
  eval_count?: number;
  total_duration?: number;
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

export interface Member {
  pid: string;
  name: string;
  email: string;
  role: "owner" | "member";
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
