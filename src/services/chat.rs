use std::time::Instant;

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::{projects, users};

use super::{
    ai::{AiConfig, AiProviderConfig, AiProviderConnection},
    ollama,
    tools::{self, ToolContext},
};

const MAX_HISTORY_MESSAGES: usize = 30;
const MAX_MESSAGE_CHARS: usize = 12_000;
const MAX_HISTORY_CHARS: usize = 60_000;
const MAX_AGENT_STEPS: usize = 8;
const MAX_TOOL_CALLS: usize = 24;
const MAX_TOOL_OUTPUT_CHARS: usize = 80_000;

const BASE_SYSTEM_PROMPT: &str = r#"Du bist der Todo-Assistent dieser Anwendung.
Nutze die bereitgestellten Tools, wenn du aktuelle Daten brauchst oder Aufgaben bearbeiten sollst. Erfinde keine IDs oder Datenbankinhalte.
Die ausgewählten Projekt-, Board- und Todo-IDs werden serverseitig ergänzt, wenn du sie in einem Tool-Aufruf weglässt.
Führe schreibende Aktionen nur aus, wenn der Nutzer sie tatsächlich verlangt. Lösche nichts aufgrund einer bloßen Vermutung.
Nach Tool-Aufrufen fasst du knapp und konkret zusammen, was du gefunden oder geändert hast. Antworte standardmäßig auf Deutsch."#;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatInputMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatInputMessage>,
    #[serde(default)]
    pub context: ToolContext,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatToolRun {
    pub name: String,
    pub success: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatResult {
    pub ok: bool,
    pub provider_id: String,
    pub model: String,
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    pub duration_ms: u64,
    pub eval_count: Option<u64>,
    pub total_duration: Option<u64>,
    pub tools: Vec<ChatToolRun>,
    pub context: ToolContext,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatProgress {
    Thinking,
    RunningTool { name: String },
}

pub async fn run<F>(
    ctx: &AppContext,
    user: &users::Model,
    request: ChatRequest,
    progress: F,
) -> Result<ChatResult>
where
    F: Fn(ChatProgress),
{
    validate_messages(&request.messages)?;
    let (context, project) = tools::validate_context(ctx, user, &request.context).await?;
    let ai_config = AiConfig::from_context(ctx)?;
    let (provider, model) = select_provider(&ai_config, project.as_ref())?;
    let system_prompt = build_system_prompt(project.as_ref(), &context);
    let mut messages = Vec::with_capacity(request.messages.len() + 8);
    messages.push(json!({ "role": "system", "content": system_prompt }));
    messages.extend(
        request
            .messages
            .into_iter()
            .map(|message| json!({ "role": message.role, "content": message.content })),
    );

    let tool_definitions = tools::ollama_definitions_for_context(&context);
    let started = Instant::now();
    let mut thinking_parts = Vec::new();
    let mut tool_runs = Vec::new();
    let mut tool_output_chars = 0;
    let mut eval_count = 0_u64;
    let mut has_eval_count = false;
    let mut total_duration = 0_u64;
    let mut has_total_duration = false;

    for _ in 0..MAX_AGENT_STEPS {
        progress(ChatProgress::Thinking);
        let generated = match &provider.connection {
            AiProviderConnection::Ollama { base_url } => {
                ollama::chat(base_url, model.clone(), &messages, &tool_definitions).await?
            }
        };
        if let Some(count) = generated.eval_count {
            eval_count = eval_count.saturating_add(count);
            has_eval_count = true;
        }
        if let Some(duration) = generated.total_duration {
            total_duration = total_duration.saturating_add(duration);
            has_total_duration = true;
        }

        let assistant_message = generated.message;
        if let Some(thinking) = assistant_message
            .get("thinking")
            .and_then(Value::as_str)
            .filter(|thinking| !thinking.trim().is_empty())
        {
            thinking_parts.push(thinking.trim().to_owned());
        }
        let calls = assistant_message
            .get("tool_calls")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let response = assistant_message
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_owned();
        messages.push(assistant_message);

        if calls.is_empty() {
            return Ok(ChatResult {
                ok: true,
                provider_id: provider.id.clone(),
                model: generated.model,
                response,
                thinking: (!thinking_parts.is_empty()).then(|| thinking_parts.join("\n\n")),
                duration_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
                eval_count: has_eval_count.then_some(eval_count),
                total_duration: has_total_duration.then_some(total_duration),
                tools: tool_runs,
                context,
            });
        }

        for call in calls {
            if tool_runs.len() >= MAX_TOOL_CALLS {
                return Err(Error::BadRequest(
                    "AI agent requested too many tool calls".into(),
                ));
            }
            let function = call
                .get("function")
                .ok_or_else(|| Error::BadRequest("Invalid Ollama tool call".into()))?;
            let name = function
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::BadRequest("Ollama tool call has no name".into()))?;
            let arguments = function
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            progress(ChatProgress::RunningTool {
                name: name.to_owned(),
            });

            let executed = tools::execute_with_context(ctx, user, name, &arguments, &context).await;
            let (success, raw_content) = match executed {
                Ok(value) => (true, model_tool_content(&value)),
                Err(error) => (false, json!({ "error": error.to_string() }).to_string()),
            };
            let remaining = MAX_TOOL_OUTPUT_CHARS.saturating_sub(tool_output_chars);
            if remaining == 0 {
                return Err(Error::BadRequest(
                    "AI agent tool output exceeded the context limit".into(),
                ));
            }
            let content = truncate_chars(raw_content, remaining);
            tool_output_chars = tool_output_chars.saturating_add(content.chars().count());
            tool_runs.push(ChatToolRun {
                name: name.to_owned(),
                success,
            });
            messages.push(json!({
                "role": "tool",
                "tool_name": name,
                "content": content,
            }));
        }
    }

    Err(Error::BadRequest(
        "AI agent did not finish within the tool-call limit".into(),
    ))
}

fn validate_messages(messages: &[ChatInputMessage]) -> Result<()> {
    if messages.is_empty() {
        return Err(Error::BadRequest(
            "At least one chat message is required".into(),
        ));
    }
    if messages.len() > MAX_HISTORY_MESSAGES {
        return Err(Error::BadRequest("Chat history is too long".into()));
    }
    if messages.last().map(|message| message.role.as_str()) != Some("user") {
        return Err(Error::BadRequest(
            "The last chat message must be from the user".into(),
        ));
    }

    let mut total_chars = 0;
    for message in messages {
        if !matches!(message.role.as_str(), "user" | "assistant") {
            return Err(Error::BadRequest("Unsupported chat message role".into()));
        }
        let chars = message.content.chars().count();
        if message.content.trim().is_empty() || chars > MAX_MESSAGE_CHARS {
            return Err(Error::BadRequest("Invalid chat message length".into()));
        }
        total_chars += chars;
    }
    if total_chars > MAX_HISTORY_CHARS {
        return Err(Error::BadRequest("Chat history is too large".into()));
    }
    Ok(())
}

fn select_provider<'a>(
    config: &'a AiConfig,
    project: Option<&projects::Model>,
) -> Result<(&'a AiProviderConfig, String)> {
    let provider_id = project
        .and_then(|project| project.ai_provider.as_deref())
        .unwrap_or(&config.default_provider);
    let provider = config.provider(provider_id)?;
    let model = project
        .and_then(|project| project.ai_model.as_deref())
        .filter(|model| !model.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| provider.default_model.clone())
        .ok_or_else(|| {
            Error::BadRequest(format!(
                "No model configured for AI provider: {}",
                provider.id
            ))
        })?;
    Ok((provider, model))
}

fn build_system_prompt(project: Option<&projects::Model>, context: &ToolContext) -> String {
    let mut prompt = BASE_SYSTEM_PROMPT.to_owned();
    if let Some(custom) = project
        .and_then(|project| project.ai_system_prompt.as_deref())
        .filter(|prompt| !prompt.trim().is_empty())
    {
        prompt.push_str("\n\nProjekt-Systemprompt:\n");
        prompt.push_str(custom.trim());
    }
    prompt
        .push_str("\n\nAktuell ausgewählter UI-Kontext (fehlende Werte sind nicht ausgewählt):\n");
    prompt.push_str(&format!(
        "project_pid: {}\nboard_pid: {}\ntodo_pid: {}",
        context.project_pid.as_deref().unwrap_or("-"),
        context.board_pid.as_deref().unwrap_or("-"),
        context.todo_pid.as_deref().unwrap_or("-")
    ));
    prompt
}

fn model_tool_content(value: &Value) -> String {
    value
        .get("content")
        .and_then(Value::as_array)
        .and_then(|content| content.first())
        .and_then(|content| content.get("text"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| value.to_string())
}

fn truncate_chars(value: String, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value;
    }
    const SUFFIX: &str = "\n… Tool-Ergebnis gekürzt …";
    let suffix_chars = SUFFIX.chars().count();
    if max_chars <= suffix_chars {
        return SUFFIX.chars().take(max_chars).collect();
    }
    let mut truncated = value
        .chars()
        .take(max_chars - suffix_chars)
        .collect::<String>();
    truncated.push_str(SUFFIX);
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_bounded_user_assistant_history() {
        assert!(validate_messages(&[
            ChatInputMessage {
                role: "user".into(),
                content: "Hallo".into(),
            },
            ChatInputMessage {
                role: "assistant".into(),
                content: "Hi".into(),
            },
            ChatInputMessage {
                role: "user".into(),
                content: "Was ist ausgewählt?".into(),
            },
        ])
        .is_ok());
        assert!(validate_messages(&[ChatInputMessage {
            role: "tool".into(),
            content: "untrusted".into(),
        }])
        .is_err());
    }

    #[test]
    fn unwraps_mcp_text_for_the_model() {
        let result = json!({ "content": [{ "type": "text", "text": "tool result" }] });
        assert_eq!(model_tool_content(&result), "tool result");
    }
}
