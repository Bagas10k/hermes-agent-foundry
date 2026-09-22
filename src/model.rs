use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub constraints: Option<AgentConstraints>,
    pub trigger: Option<TriggerConfig>,
    pub modules: Vec<ModuleSpec>,
    pub pipeline_dag: PipelineDag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConstraints {
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default = "default_token_budget")]
    pub token_budget_per_run: u64,
    #[serde(default)]
    pub requires_human_approval: bool,
}

fn default_max_steps() -> usize { 10 }
fn default_timeout_seconds() -> u64 { 180 }
fn default_token_budget() -> u64 { 15000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(rename = "type")]
    pub trigger_type: String, // "cron", "webhook", "manual"
    pub schedule: Option<String>,
    pub timezone: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSpec {
    pub id: String,
    #[serde(rename = "type")]
    pub module_type: String, // "trigger", "context", "reasoning", "tool", "guardrail", "output"
    pub provider: String,
    pub name: Option<String>,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDag {
    pub nodes: Vec<String>,
    pub edges: Vec<DagEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagEdge {
    pub from: String,
    pub to: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRun {
    pub id: String,
    pub agent_id: String,
    pub status: String, // "PENDING", "RUNNING", "COMPLETED", "FAILED", "ABORTED"
    pub started_at: String,
    pub completed_at: Option<String>,
    pub steps_executed: usize,
    pub tokens_consumed: u64,
    pub output_summary: Option<String>,
    pub error_message: Option<String>,
}
