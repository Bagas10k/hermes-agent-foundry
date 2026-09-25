// modules/tools/http_fetch.rs
// Hermes Agent Foundry :: HTTP Fetch Tool Module
// Hak Cipta: Bagas Cihuy (Bagas Saputra)

use serde::{Deserialize, Serialize};
use std::time::Instant;
use crate::modules::tools::ToolCallResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpFetchRequest {
    pub url: String,
    pub method: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: Option<String>,
}

pub struct HttpFetchTool {
    client: reqwest::Client,
    max_response_bytes: usize,
}

impl Default for HttpFetchTool {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            client,
            max_response_bytes: 2 * 1024 * 1024, // 2MB max response
        }
    }
}

impl HttpFetchTool {
    pub fn new(timeout_secs: u64, max_response_bytes: usize) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_default();
        Self {
            client,
            max_response_bytes,
        }
    }

    /// Validasi skema URL untuk mencegah SSRF ke cloud internal metadata service
    pub fn validate_safety(url: &str) -> Result<(), String> {
        let lower = url.to_lowercase();
        if lower.starts_with("http://169.254.169.254") || lower.starts_with("http://metadata.google.internal") {
            return Err("Akses ke AWS/Cloud Metadata service diblokir untuk mencegah SSRF.".to_string());
        }
        if !lower.starts_with("http://") && !lower.starts_with("https://") {
            return Err("Hanya protokol http:// dan https:// yang diizinkan.".to_string());
        }
        Ok(())
    }

    /// Eksekusi HTTP Request
    pub async fn execute(&self, req: HttpFetchRequest) -> ToolCallResult {
        let start = Instant::now();

        if let Err(e) = Self::validate_safety(&req.url) {
            return ToolCallResult {
                success: false,
                output: String::new(),
                error: Some(e),
                execution_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        let method = match req.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            _ => reqwest::Method::GET,
        };

        let mut req_builder = self.client.request(method, &req.url);

        if let Some(headers) = req.headers {
            for (k, v) in headers {
                req_builder = req_builder.header(k, v);
            }
        }

        if let Some(body) = req.body {
            req_builder = req_builder.body(body);
        }

        match req_builder.send().await {
            Ok(resp) => {
                let status = resp.status();
                let status_code = status.as_u16();
                match resp.text().await {
                    Ok(text) => {
                        let truncated = if text.len() > self.max_response_bytes {
                            format!("{}... [DIPANGKAS: Melebihi {} bytes]", &text[..self.max_response_bytes], self.max_response_bytes)
                        } else {
                            text
                        };

                        let execution_time_ms = start.elapsed().as_millis() as u64;
                        let json_res = serde_json::json!({
                            "status_code": status_code,
                            "body": truncated
                        });

                        ToolCallResult {
                            success: status.is_success(),
                            output: serde_json::to_string_pretty(&json_res).unwrap_or_default(),
                            error: if !status.is_success() { Some(format!("HTTP error status: {}", status_code)) } else { None },
                            execution_time_ms,
                        }
                    }
                    Err(e) => ToolCallResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Gagal membaca body response: {}", e)),
                        execution_time_ms: start.elapsed().as_millis() as u64,
                    }
                }
            }
            Err(e) => ToolCallResult {
                success: false,
                output: String::new(),
                error: Some(format!("Gagal melakukan request network: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssrf_prevention() {
        assert!(HttpFetchTool::validate_safety("http://169.254.169.254/latest/meta-data/").is_err());
        assert!(HttpFetchTool::validate_safety("http://metadata.google.internal/computeMetadata/v1/").is_err());
        assert!(HttpFetchTool::validate_safety("https://api.github.com/zen").is_ok());
    }

    #[tokio::test]
    async fn test_http_fetch_blocked_ssrf() {
        let tool = HttpFetchTool::default();
        let res = tool.execute(HttpFetchRequest {
            url: "http://169.254.169.254/secret".to_string(),
            method: "GET".to_string(),
            headers: None,
            body: None,
        }).await;
        assert!(!res.success);
        assert!(res.error.unwrap().contains("SSRF"));
    }
}
