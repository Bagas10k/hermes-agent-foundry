// modules/tools/mod.rs
// Hermes Agent Foundry :: Modular Toolkits & Sandboxed Execution
// Hak Cipta: Bagas Cihuy (Bagas Saputra)

pub mod web_search;
pub mod terminal_exec;
pub mod browser_bridge;
pub mod http_fetch;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}
