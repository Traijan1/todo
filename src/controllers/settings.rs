#![allow(clippy::missing_errors_doc)]

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::ai;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderParams {
    pub provider_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestPromptParams {
    pub provider_id: String,
    pub prompt: String,
    pub model: String,
    pub system_prompt: Option<String>,
}

#[debug_handler]
pub async fn get_settings(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let config = ai::AiConfig::from_context(&ctx)?;
    format::json(config.catalog())
}

#[debug_handler]
pub async fn fetch_models(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<ProviderParams>,
) -> Result<Response> {
    let config = ai::AiConfig::from_context(&ctx)?;
    let provider = config.provider(params.provider_id.trim())?;
    let models = ai::list_models(provider).await?;

    format::json(serde_json::json!({
        "ok": true,
        "provider_id": provider.id,
        "models": models,
    }))
}

#[debug_handler]
pub async fn test_prompt(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<TestPromptParams>,
) -> Result<Response> {
    if params.prompt.trim().is_empty() {
        return Err(Error::BadRequest("Prompt cannot be empty".into()));
    }
    if params.model.trim().is_empty() {
        return Err(Error::BadRequest("Model cannot be empty".into()));
    }

    let config = ai::AiConfig::from_context(&ctx)?;
    let provider = config.provider(params.provider_id.trim())?;
    let result = ai::generate(
        provider,
        params.model,
        &params.prompt,
        params.system_prompt.as_deref(),
    )
    .await?;

    format::json(result)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/settings/")
        .add("/", get(get_settings))
        .add("ai/models", post(fetch_models))
        .add("ai/test-prompt", post(test_prompt))
}
