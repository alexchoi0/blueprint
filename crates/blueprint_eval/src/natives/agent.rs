use std::collections::HashMap;
use std::sync::Arc;

use blueprint_core::{BlueprintError, NativeFunction, Result, Value};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::eval::Evaluator;

pub fn register(evaluator: &mut Evaluator) {
    evaluator.register_native(NativeFunction::new("agent", agent));
}

async fn agent(args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(BlueprintError::ArgumentError {
            message: format!("agent() takes 1 or 2 arguments ({} given)", args.len()),
        });
    }

    let prompt = args[0].as_string()?;

    let system = if args.len() == 2 {
        Some(args[1].as_string()?)
    } else {
        kwargs
            .get("system")
            .map(|v| v.as_string())
            .transpose()?
    };

    let model = kwargs
        .get("model")
        .map(|v| v.as_string())
        .transpose()?
        .unwrap_or_else(|| "gpt-4o".to_string());

    let temperature = kwargs
        .get("temperature")
        .map(|v| v.as_float())
        .transpose()?
        .unwrap_or(0.7);

    let api_key = kwargs
        .get("api_key")
        .map(|v| v.as_string())
        .transpose()?;

    if model.starts_with("claude") {
        call_anthropic(&prompt, system.as_deref(), &model, temperature, api_key.as_deref()).await
    } else {
        call_openai(&prompt, system.as_deref(), &model, temperature, api_key.as_deref()).await
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f64,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Deserialize)]
struct OpenAIMessageResponse {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
}

async fn call_openai(
    prompt: &str,
    system: Option<&str>,
    model: &str,
    temperature: f64,
    api_key: Option<&str>,
) -> Result<Value> {
    let key = api_key
        .map(|s| s.to_string())
        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        .ok_or_else(|| BlueprintError::ArgumentError {
            message: "OPENAI_API_KEY not set and no api_key provided".into(),
        })?;

    let mut messages = Vec::new();
    if let Some(sys) = system {
        messages.push(OpenAIMessage {
            role: "system".into(),
            content: sys.into(),
        });
    }
    messages.push(OpenAIMessage {
        role: "user".into(),
        content: prompt.into(),
    });

    let request = OpenAIRequest {
        model: model.into(),
        messages,
        temperature,
    };

    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| BlueprintError::HttpError {
            url: "https://api.openai.com/v1/chat/completions".into(),
            message: e.to_string(),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(BlueprintError::HttpError {
            url: "https://api.openai.com/v1/chat/completions".into(),
            message: format!("HTTP {}: {}", status, body),
        });
    }

    let resp: OpenAIResponse = response.json().await.map_err(|e| BlueprintError::HttpError {
        url: "https://api.openai.com/v1/chat/completions".into(),
        message: e.to_string(),
    })?;

    let content = resp
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default();

    let mut result = HashMap::new();
    result.insert("content".to_string(), Value::String(Arc::new(content)));
    result.insert(
        "model".to_string(),
        Value::String(Arc::new(resp.model.unwrap_or_else(|| model.to_string()))),
    );

    if let Some(usage) = resp.usage {
        let mut tokens = HashMap::new();
        tokens.insert("prompt".to_string(), Value::Int(usage.prompt_tokens));
        tokens.insert("completion".to_string(), Value::Int(usage.completion_tokens));
        tokens.insert("total".to_string(), Value::Int(usage.total_tokens));
        result.insert(
            "tokens".to_string(),
            Value::Dict(Arc::new(RwLock::new(tokens))),
        );
    }

    Ok(Value::Dict(Arc::new(RwLock::new(result))))
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: i64,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    temperature: f64,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: i64,
    output_tokens: i64,
}

async fn call_anthropic(
    prompt: &str,
    system: Option<&str>,
    model: &str,
    temperature: f64,
    api_key: Option<&str>,
) -> Result<Value> {
    let key = api_key
        .map(|s| s.to_string())
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .ok_or_else(|| BlueprintError::ArgumentError {
            message: "ANTHROPIC_API_KEY not set and no api_key provided".into(),
        })?;

    let request = AnthropicRequest {
        model: model.into(),
        max_tokens: 4096,
        messages: vec![AnthropicMessage {
            role: "user".into(),
            content: prompt.into(),
        }],
        system: system.map(|s| s.to_string()),
        temperature,
    };

    let client = Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| BlueprintError::HttpError {
            url: "https://api.anthropic.com/v1/messages".into(),
            message: e.to_string(),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(BlueprintError::HttpError {
            url: "https://api.anthropic.com/v1/messages".into(),
            message: format!("HTTP {}: {}", status, body),
        });
    }

    let resp: AnthropicResponse = response.json().await.map_err(|e| BlueprintError::HttpError {
        url: "https://api.anthropic.com/v1/messages".into(),
        message: e.to_string(),
    })?;

    let content = resp
        .content
        .first()
        .and_then(|c| c.text.clone())
        .unwrap_or_default();

    let mut result = HashMap::new();
    result.insert("content".to_string(), Value::String(Arc::new(content)));
    result.insert(
        "model".to_string(),
        Value::String(Arc::new(resp.model.unwrap_or_else(|| model.to_string()))),
    );

    if let Some(usage) = resp.usage {
        let mut tokens = HashMap::new();
        tokens.insert("prompt".to_string(), Value::Int(usage.input_tokens));
        tokens.insert("completion".to_string(), Value::Int(usage.output_tokens));
        tokens.insert("total".to_string(), Value::Int(usage.input_tokens + usage.output_tokens));
        result.insert(
            "tokens".to_string(),
            Value::Dict(Arc::new(RwLock::new(tokens))),
        );
    }

    Ok(Value::Dict(Arc::new(RwLock::new(result))))
}
