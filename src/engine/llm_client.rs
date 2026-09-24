use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<Usage>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LlmClient {
    client: Client,
    pub base_url: String,
    pub api_key: Option<String>,
    pub default_model: String,
}

#[allow(dead_code)]
impl LlmClient {
    pub fn new(base_url: String, api_key: Option<String>, default_model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(5)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            default_model,
        }
    }

    pub fn for_9router() -> Self {
        let base_url = std::env::var("LLM_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:20128/v1".to_string());
        let api_key = std::env::var("LLM_API_KEY").ok();
        let default_model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        Self::new(base_url, api_key, default_model)
    }

    pub async fn chat_completion(&self, request: &ChatCompletionRequest) -> Result<ChatCompletionResponse, String> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut req = self.client.post(&url).json(request);

        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await.map_err(|e| format!("HTTP request error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("LLM Gateway error ({status}): {body}"));
        }

        let completion: ChatCompletionResponse = resp.json().await.map_err(|e| format!("JSON decode error: {e}"))?;
        Ok(completion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_init() {
        let client = LlmClient::new("http://localhost:20128/v1/".to_string(), Some("test-key".to_string()), "gpt-4o".to_string());
        assert_eq!(client.base_url, "http://localhost:20128/v1");
        assert_eq!(client.default_model, "gpt-4o");
    }

    #[test]
    fn test_request_serialization() {
        let req = ChatCompletionRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Halo Hermes".to_string(),
            }],
            temperature: Some(0.3),
            max_tokens: Some(100),
            stream: false,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("gpt-4o-mini"));
        assert!(json.contains("temperature"));
    }
}
