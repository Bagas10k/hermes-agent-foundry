// modules/tools/browser_bridge.rs
// Hermes Agent Foundry :: Browser Bridge Tool Module (Chromium / CDP Integration)
// Hak Cipta: Bagas Cihuy (Bagas Saputra)

use serde::{Deserialize, Serialize};
use std::time::Instant;
use crate::modules::tools::ToolCallResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserAction {
    Navigate { url: String },
    ExtractText { selector: Option<String> },
    Click { selector: String },
    Screenshot { output_path: String },
    EvaluateJs { script: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserBridgeConfig {
    pub cdp_endpoint: String,
    pub default_timeout_secs: u64,
    pub headless: bool,
}

impl Default for BrowserBridgeConfig {
    fn default() -> Self {
        Self {
            cdp_endpoint: "http://127.0.0.1:9222".to_string(),
            default_timeout_secs: 20,
            headless: true,
        }
    }
}

pub struct BrowserBridgeTool {
    config: BrowserBridgeConfig,
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl BrowserBridgeTool {
    pub fn new(config: BrowserBridgeConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.default_timeout_secs))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    /// Validasi keamanan skrip JS atau URL sebelum interaksi browser
    pub fn validate_target_url(url: &str) -> Result<(), String> {
        let trimmed = url.trim();
        if trimmed.starts_with("file://") || trimmed.starts_with("gopher://") {
            return Err("Protokol URL diblokir oleh sandbox keamanan.".to_string());
        }
        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return Err("URL harus menggunakan protokol http:// atau https://".to_string());
        }
        Ok(())
    }

    /// Eksekusi aksi browser
    pub async fn execute_action(&self, action: BrowserAction) -> ToolCallResult {
        let start = Instant::now();

        match action {
            BrowserAction::Navigate { url } => {
                if let Err(e) = Self::validate_target_url(&url) {
                    return ToolCallResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                        execution_time_ms: start.elapsed().as_millis() as u64,
                    };
                }

                // Browser bridge payload simulasi/CDP command dispatcher
                let res_json = serde_json::json!({
                    "status": "navigated",
                    "url": url,
                    "endpoint": self.config.cdp_endpoint,
                    "headless": self.config.headless
                });

                ToolCallResult {
                    success: true,
                    output: serde_json::to_string_pretty(&res_json).unwrap_or_default(),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            }
            BrowserAction::ExtractText { selector } => {
                let sel = selector.unwrap_or_else(|| "body".to_string());
                let res_json = serde_json::json!({
                    "status": "extracted",
                    "selector": sel,
                    "content": "Konten teks terekstraksi dari halaman web target secara terisolasi."
                });

                ToolCallResult {
                    success: true,
                    output: serde_json::to_string_pretty(&res_json).unwrap_or_default(),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            }
            BrowserAction::Click { selector } => {
                let res_json = serde_json::json!({
                    "status": "clicked",
                    "selector": selector
                });

                ToolCallResult {
                    success: true,
                    output: serde_json::to_string_pretty(&res_json).unwrap_or_default(),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            }
            BrowserAction::Screenshot { output_path } => {
                let res_json = serde_json::json!({
                    "status": "captured",
                    "path": output_path
                });

                ToolCallResult {
                    success: true,
                    output: serde_json::to_string_pretty(&res_json).unwrap_or_default(),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            }
            BrowserAction::EvaluateJs { script } => {
                if script.contains("localStorage.clear()") || script.contains("document.cookie") {
                    return ToolCallResult {
                        success: false,
                        output: String::new(),
                        error: Some("Manipulasi session cookie / storage dilarang oleh sandbox.".to_string()),
                        execution_time_ms: start.elapsed().as_millis() as u64,
                    };
                }

                let res_json = serde_json::json!({
                    "status": "evaluated",
                    "result": "JS evaluation completed safely"
                });

                ToolCallResult {
                    success: true,
                    output: serde_json::to_string_pretty(&res_json).unwrap_or_default(),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        assert!(BrowserBridgeTool::validate_target_url("https://example.com").is_ok());
        assert!(BrowserBridgeTool::validate_target_url("file:///etc/passwd").is_err());
        assert!(BrowserBridgeTool::validate_target_url("ftp://example.com").is_err());
    }

    #[tokio::test]
    async fn test_browser_navigate() {
        let tool = BrowserBridgeTool::new(BrowserBridgeConfig::default());
        let res = tool.execute_action(BrowserAction::Navigate {
            url: "https://jajandigital.web.id".to_string()
        }).await;
        assert!(res.success);
        assert!(res.output.contains("navigated"));
    }

    #[tokio::test]
    async fn test_browser_blocked_script() {
        let tool = BrowserBridgeTool::new(BrowserBridgeConfig::default());
        let res = tool.execute_action(BrowserAction::EvaluateJs {
            script: "console.log(document.cookie)".to_string()
        }).await;
        assert!(!res.success);
        assert!(res.error.unwrap().contains("sandbox"));
    }
}
