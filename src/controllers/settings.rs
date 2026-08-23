#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{user_settings, users};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSettingsParams {
    pub ollama_url: String,
    pub default_model: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchModelsParams {
    pub ollama_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestPromptParams {
    pub prompt: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub ollama_url: Option<String>,
}

#[debug_handler]
pub async fn get_settings(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let settings = user_settings::Model::get_or_default(&ctx.db, user.id).await?;
    format::json(settings)
}

#[debug_handler]
pub async fn update_settings(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateSettingsParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let ollama_url = params.ollama_url.trim().to_string();
    if ollama_url.is_empty() {
        return Err(Error::BadRequest("ollama_url cannot be empty".into()));
    }

    let saved =
        user_settings::Model::upsert(&ctx.db, user.id, ollama_url, params.default_model).await?;

    format::json(saved)
}

#[debug_handler]
pub async fn fetch_ollama_models(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<FetchModelsParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let settings = user_settings::Model::get_or_default(&ctx.db, user.id).await?;

    let base_url = params
        .ollama_url
        .filter(|u| !u.trim().is_empty())
        .unwrap_or(settings.ollama_url);
    if base_url.trim().is_empty() {
        return Err(Error::BadRequest("Ollama URL is not configured".into()));
    }
    let clean_url = base_url.trim_end_matches('/');

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| Error::InternalServerError)?;

    let url = format!("{clean_url}/api/tags");
    let res =
        client.get(&url).send().await.map_err(|e| {
            Error::BadRequest(format!("Failed to reach Ollama at {clean_url}: {e}"))
        })?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(Error::BadRequest(format!(
            "Ollama returned error: {err_text}"
        )));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to parse response: {e}")))?;

    format::json(serde_json::json!({
        "ok": true,
        "ollama_url": clean_url,
        "models": json.get("models").unwrap_or(&serde_json::json!([])),
    }))
}

#[debug_handler]
pub async fn test_ollama_prompt(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<TestPromptParams>,
) -> Result<Response> {
    if params.prompt.trim().is_empty() {
        return Err(Error::BadRequest("Prompt cannot be empty".into()));
    }
    if params.model.trim().is_empty() {
        return Err(Error::BadRequest("Model cannot be empty".into()));
    }

    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let settings = user_settings::Model::get_or_default(&ctx.db, user.id).await?;

    let base_url = params
        .ollama_url
        .filter(|u| !u.trim().is_empty())
        .unwrap_or(settings.ollama_url);
    if base_url.trim().is_empty() {
        return Err(Error::BadRequest("Ollama URL is not configured".into()));
    }
    let clean_url = base_url.trim_end_matches('/');

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|_| Error::InternalServerError)?;

    let mut body = serde_json::json!({
        "model": params.model,
        "prompt": params.prompt,
        "stream": false,
    });

    if let Some(sys) = &params.system_prompt {
        if !sys.trim().is_empty() {
            body["system"] = serde_json::json!(sys);
        }
    }

    let start_time = std::time::Instant::now();
    let url = format!("{clean_url}/api/generate");
    let res =
        client.post(&url).json(&body).send().await.map_err(|e| {
            Error::BadRequest(format!("Failed to reach Ollama at {clean_url}: {e}"))
        })?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(Error::BadRequest(format!(
            "Ollama generation error: {err_text}"
        )));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to parse response: {e}")))?;

    let duration_ms = start_time.elapsed().as_millis();
    let response_text = json
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    format::json(serde_json::json!({
        "ok": true,
        "model": params.model,
        "response": response_text,
        "duration_ms": duration_ms,
        "eval_count": json.get("eval_count"),
        "total_duration": json.get("total_duration"),
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/settings/")
        .add("/", get(get_settings))
        .add("/", put(update_settings))
        .add("ollama/models", post(fetch_ollama_models))
        .add("ollama/test-prompt", post(test_ollama_prompt))
}
