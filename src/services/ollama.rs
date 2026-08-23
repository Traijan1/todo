use loco_rs::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GenerateResult {
    pub ok: bool,
    pub model: String,
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    pub duration_ms: u64,
    pub eval_count: Option<u64>,
    pub total_duration: Option<u64>,
}

pub async fn generate(
    base_url: &str,
    model: String,
    prompt: &str,
    system_prompt: Option<&str>,
) -> Result<GenerateResult> {
    let clean_url = base_url.trim().trim_end_matches('/');
    if clean_url.is_empty() {
        return Err(Error::BadRequest("Ollama URL is not configured".into()));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|_| Error::InternalServerError)?;

    let mut body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "think": true,
    });

    if let Some(system_prompt) = system_prompt.filter(|value| !value.trim().is_empty()) {
        body["system"] = serde_json::json!(system_prompt);
    }

    let url = format!("{clean_url}/api/generate");
    let start_time = std::time::Instant::now();
    let mut response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|error| {
            Error::BadRequest(format!("Failed to reach Ollama at {clean_url}: {error}"))
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        // Models without thinking support should still be usable. Ollama rejects
        // `think: true` immediately for those models, so retry once without it.
        if error_text.to_lowercase().contains("think") {
            body.as_object_mut().map(|object| object.remove("think"));
            response = client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|error| {
                    Error::BadRequest(format!("Failed to reach Ollama at {clean_url}: {error}"))
                })?;
        } else {
            return Err(Error::BadRequest(format!(
                "Ollama returned {status}: {error_text}"
            )));
        }
    }

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(Error::BadRequest(format!(
            "Ollama returned {status}: {error_text}"
        )));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|error| Error::BadRequest(format!("Failed to parse Ollama response: {error}")))?;

    let thinking = json
        .get("thinking")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string);

    Ok(GenerateResult {
        ok: true,
        model,
        response: json
            .get("response")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        thinking,
        duration_ms: start_time
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX),
        eval_count: json.get("eval_count").and_then(serde_json::Value::as_u64),
        total_duration: json
            .get("total_duration")
            .and_then(serde_json::Value::as_u64),
    })
}
