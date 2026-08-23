import type { AiRequestContext } from "../stores/aiContext";

export interface ChatTransportMessage {
  role: "user" | "assistant";
  content: string;
}

export interface ChatToolRun {
  name: string;
  success: boolean;
}

export interface ChatResult {
  ok: boolean;
  provider_id: string;
  model: string;
  response: string;
  thinking?: string;
  duration_ms: number;
  eval_count?: number;
  total_duration?: number;
  tools: ChatToolRun[];
  context: AiRequestContext;
}

export type ChatStreamEvent =
  | {
      type: "progress";
      progress: { kind: "thinking" } | { kind: "running_tool"; name: string };
    }
  | { type: "done"; result: ChatResult }
  | { type: "error"; message: string };

export async function streamChat(
  messages: ChatTransportMessage[],
  context: AiRequestContext,
  onEvent: (event: ChatStreamEvent) => void,
  signal?: AbortSignal,
) {
  const token = localStorage.getItem("token");
  const response = await fetch("/api/chat", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify({ messages, context }),
    signal,
  });

  if (!response.ok) {
    const body = await response.text();
    let message = body || `Request failed with status ${response.status}`;
    try {
      const parsed = JSON.parse(body);
      message = parsed.description || parsed.message || message;
    } catch {}
    throw new Error(message);
  }
  if (!response.body) {
    throw new Error("Der Browser unterstützt keinen Chat-Stream.");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  const consume = (block: string) => {
    const data = block
      .split(/\r?\n/)
      .filter((line) => line.startsWith("data:"))
      .map((line) => line.slice(5).trimStart())
      .join("\n");
    if (data) onEvent(JSON.parse(data) as ChatStreamEvent);
  };

  while (true) {
    const { done, value } = await reader.read();
    buffer += decoder.decode(value, { stream: !done });
    const blocks = buffer.split(/\r?\n\r?\n/);
    buffer = blocks.pop() || "";
    for (const block of blocks) consume(block);
    if (done) break;
  }
  if (buffer.trim()) consume(buffer);
}
