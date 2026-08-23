use std::collections::HashSet;

use loco_rs::{app::AppContext, prelude::*};
use serde::{Deserialize, Serialize};

use super::ollama;

#[derive(Clone, Debug, Deserialize)]
pub struct AiConfig {
    pub default_provider: String,
    pub providers: Vec<AiProviderConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AiProviderConfig {
    pub id: String,
    pub name: String,
    pub default_model: Option<String>,
    #[serde(flatten)]
    pub connection: AiProviderConnection,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AiProviderConnection {
    Ollama { base_url: String },
}

#[derive(Clone, Debug, Serialize)]
pub struct AiCatalog {
    pub default_provider: String,
    pub providers: Vec<AiProviderSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiProviderSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub default_model: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiModel {
    pub id: String,
    pub name: String,
    pub size: Option<u64>,
    pub details: Option<AiModelDetails>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiModelDetails {
    pub family: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateResult {
    pub ok: bool,
    pub provider_id: String,
    pub model: String,
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    pub duration_ms: u64,
    pub eval_count: Option<u64>,
    pub total_duration: Option<u64>,
}

#[derive(Debug)]
pub struct AdapterGenerateResult {
    pub model: String,
    pub response: String,
    pub thinking: Option<String>,
    pub duration_ms: u64,
    pub eval_count: Option<u64>,
    pub total_duration: Option<u64>,
}

impl AiProviderConfig {
    pub fn kind(&self) -> &'static str {
        match &self.connection {
            AiProviderConnection::Ollama { .. } => "ollama",
        }
    }
}

impl From<&AiProviderConfig> for AiProviderSummary {
    fn from(provider: &AiProviderConfig) -> Self {
        Self {
            id: provider.id.clone(),
            name: provider.name.clone(),
            kind: provider.kind().to_string(),
            default_model: provider.default_model.clone(),
        }
    }
}

impl AiConfig {
    pub fn from_context(ctx: &AppContext) -> Result<Self> {
        let Some(value) = ctx
            .config
            .settings
            .as_ref()
            .and_then(|settings| settings.get("ai"))
            .cloned()
        else {
            tracing::error!("missing settings.ai configuration");
            return Err(Error::InternalServerError);
        };

        let config: Self = serde_json::from_value(value).map_err(|error| {
            tracing::error!(%error, "invalid settings.ai configuration");
            Error::InternalServerError
        })?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.providers.is_empty() {
            tracing::error!("settings.ai.providers must not be empty");
            return Err(Error::InternalServerError);
        }

        let mut ids = HashSet::new();
        for provider in &self.providers {
            if provider.id.trim().is_empty()
                || provider.name.trim().is_empty()
                || match &provider.connection {
                    AiProviderConnection::Ollama { base_url } => base_url.trim().is_empty(),
                }
            {
                tracing::error!(provider_id = %provider.id, "AI provider has empty required fields");
                return Err(Error::InternalServerError);
            }
            if !ids.insert(provider.id.as_str()) {
                tracing::error!(provider_id = %provider.id, "duplicate AI provider id");
                return Err(Error::InternalServerError);
            }
        }

        if !ids.contains(self.default_provider.as_str()) {
            tracing::error!(
                default_provider = %self.default_provider,
                "default AI provider does not exist"
            );
            return Err(Error::InternalServerError);
        }

        Ok(())
    }

    pub fn catalog(&self) -> AiCatalog {
        AiCatalog {
            default_provider: self.default_provider.clone(),
            providers: self.providers.iter().map(Into::into).collect(),
        }
    }

    pub fn provider(&self, id: &str) -> Result<&AiProviderConfig> {
        self.providers
            .iter()
            .find(|provider| provider.id == id)
            .ok_or_else(|| Error::BadRequest(format!("Unknown AI provider: {id}")))
    }
}

pub async fn list_models(provider: &AiProviderConfig) -> Result<Vec<AiModel>> {
    match &provider.connection {
        AiProviderConnection::Ollama { base_url } => ollama::list_models(base_url).await,
    }
}

pub async fn generate(
    provider: &AiProviderConfig,
    model: String,
    prompt: &str,
    system_prompt: Option<&str>,
) -> Result<GenerateResult> {
    let generated = match &provider.connection {
        AiProviderConnection::Ollama { base_url } => {
            ollama::generate(base_url, model, prompt, system_prompt).await
        }
    }?;

    Ok(GenerateResult {
        ok: true,
        provider_id: provider.id.clone(),
        model: generated.model,
        response: generated.response,
        thinking: generated.thinking,
        duration_ms: generated.duration_ms,
        eval_count: generated.eval_count,
        total_duration: generated.total_duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> AiConfig {
        AiConfig {
            default_provider: "local".into(),
            providers: vec![AiProviderConfig {
                id: "local".into(),
                name: "Local Ollama".into(),
                default_model: Some("llama3.2".into()),
                connection: AiProviderConnection::Ollama {
                    base_url: "http://localhost:11434".into(),
                },
            }],
        }
    }

    #[test]
    fn validates_and_resolves_provider_catalog() {
        let config = valid_config();
        assert!(config.validate().is_ok());
        assert_eq!(config.provider("local").unwrap().kind(), "ollama");
        assert_eq!(config.catalog().providers[0].id, "local");
    }

    #[test]
    fn rejects_duplicate_and_missing_default_providers() {
        let mut duplicate = valid_config();
        duplicate.providers.push(duplicate.providers[0].clone());
        assert!(duplicate.validate().is_err());

        let mut missing_default = valid_config();
        missing_default.default_provider = "missing".into();
        assert!(missing_default.validate().is_err());
    }

    #[test]
    fn deserializes_the_documented_provider_shape() {
        let config: AiConfig = serde_json::from_value(serde_json::json!({
            "default_provider": "ollama",
            "providers": [{
                "id": "ollama",
                "name": "Local Ollama",
                "kind": "ollama",
                "base_url": "http://host.docker.internal:11434",
                "default_model": "qwen3.8:27b"
            }]
        }))
        .unwrap();

        assert!(config.validate().is_ok());
        assert_eq!(config.provider("ollama").unwrap().kind(), "ollama");
    }
}
