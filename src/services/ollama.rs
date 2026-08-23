use super::ai::{AdapterGenerateResult, AiModel, AiModelDetails};
use loco_rs::prelude::*;

pub async fn list_models(base_url: &str) -> Result<Vec<AiModel>> {
    let clean_url = base_url.trim().trim_end_matches('/');
    if clean_url.is_empty() {
        return Err(Error::BadRequest("Ollama URL is not configured".into()));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| Error::InternalServerError)?;

    let response = client
        .get(format!("{clean_url}/api/tags"))
        .send()
        .await
        .map_err(|error| Error::BadRequest(format!("Failed to reach Ollama: {error}")))?;

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

    Ok(json
        .get("models")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|model| {
            let name = model
                .get("name")
                .or_else(|| model.get("model"))?
                .as_str()?
                .to_string();
            let details = model.get("details").map(|details| AiModelDetails {
                family: details
                    .get("family")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string),
                parameter_size: details
                    .get("parameter_size")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string),
                quantization_level: details
                    .get("quantization_level")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string),
            });
            Some(AiModel {
                id: name.clone(),
                name,
                size: model.get("size").and_then(serde_json::Value::as_u64),
                details,
            })
        })
        .collect())
}

pub async fn generate(
    base_url: &str,
    model: String,
    prompt: &str,
    system_prompt: Option<&str>,
) -> Result<AdapterGenerateResult> {
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
        .map_err(|error| Error::BadRequest(format!("Failed to reach Ollama provider: {error}")))?;

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
                    Error::BadRequest(format!("Failed to reach Ollama provider: {error}"))
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

    Ok(AdapterGenerateResult {
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
