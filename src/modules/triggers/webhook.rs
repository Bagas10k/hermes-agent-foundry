use serde::{Deserialize, Serialize};
use serde_json::json;

/// Konfigurasi pemicu Webhook Inbound
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTriggerConfig {
    pub endpoint_path: Option<String>,
    pub secret_token: Option<String>,
    pub allowed_sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTriggerEvent {
    pub received_at: String,
    pub event_type: String,
    pub payload_hash: String,
    pub payload: serde_json::Value,
    pub verified: bool,
}

pub struct WebhookTriggerHandler;

impl WebhookTriggerHandler {
    pub fn parse_config(params: &serde_json::Value) -> Result<WebhookTriggerConfig, String> {
        let endpoint_path = params.get("endpoint_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let secret_token = params.get("secret_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let allowed_sources = params.get("allowed_sources")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect());

        Ok(WebhookTriggerConfig {
            endpoint_path,
            secret_token,
            allowed_sources,
        })
    }

    pub fn evaluate(
        config: &WebhookTriggerConfig,
        incoming_payload: Option<&serde_json::Value>,
        headers_token: Option<&str>,
    ) -> (serde_json::Value, u64) {
        let now = chrono::Utc::now();
        let payload = incoming_payload.cloned().unwrap_or(json!({
            "event": "ping",
            "message": "Webhook default test signal"
        }));

        // Validasi secret_token bila dikonfigurasi
        let verified = match &config.secret_token {
            Some(token) => {
                if let Some(hdr) = headers_token {
                    hdr == token
                } else {
                    false
                }
            }
            None => true,
        };

        use sha2::{Digest, Sha256};
        let payload_str = serde_json::to_string(&payload).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(payload_str.as_bytes());
        let payload_hash = format!("{:x}", hasher.finalize());

        let event = WebhookTriggerEvent {
            received_at: now.to_rfc3339(),
            event_type: payload.get("event").and_then(|e| e.as_str()).unwrap_or("generic_webhook").to_string(),
            payload_hash,
            payload,
            verified,
        };

        (json!(event), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_trigger_eval() {
        let config = WebhookTriggerConfig {
            endpoint_path: Some("/webhooks/trading-alert".to_string()),
            secret_token: Some("secret123".to_string()),
            allowed_sources: None,
        };

        let payload = json!({
            "event": "PRICE_BREAKOUT",
            "symbol": "BTC/USDT",
            "price": 89450.0
        });

        // Kasus 1: Token valid
        let (output, tokens) = WebhookTriggerHandler::evaluate(&config, Some(&payload), Some("secret123"));
        assert_eq!(tokens, 0);
        assert_eq!(output["verified"], true);
        assert_eq!(output["event_type"], "PRICE_BREAKOUT");

        // Kasus 2: Token salah
        let (output_invalid, _) = WebhookTriggerHandler::evaluate(&config, Some(&payload), Some("wrong"));
        assert_eq!(output_invalid["verified"], false);
    }
}
