use crate::circuit_breaker::CircuitBreaker;
use crate::model::{AgentConstraints, AgentSpec, ModuleSpec};
use crate::dag::validate_and_sort_dag;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

/// Representasi konteks memori eksekusi ephemeral (in-memory, zero GC overhead)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerContext {
    pub run_id: String,
    pub agent_id: String,
    pub state_data: HashMap<String, serde_json::Value>,
    pub step_logs: Vec<StepLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepLog {
    pub node_id: String,
    pub module_type: String,
    pub provider: String,
    pub status: String,
    pub duration_us: u128,
    pub output_snippet: String,
    pub tokens_consumed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerExecutionReport {
    pub run_id: String,
    pub agent_id: String,
    pub status: String, // "COMPLETED", "FAILED", "CIRCUIT_BROKEN"
    pub execution_order: Vec<String>,
    pub steps_executed: usize,
    pub total_duration_us: u128,
    pub total_tokens_consumed: u64,
    pub output_summary: String,
    pub step_logs: Vec<StepLog>,
    pub final_state: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
}

pub struct EphemeralWorker {
    pub spec: AgentSpec,
    pub context: WorkerContext,
    pub circuit_breaker: CircuitBreaker,
    pub modules_map: HashMap<String, ModuleSpec>,
}

impl EphemeralWorker {
    pub fn new(spec: AgentSpec, run_id: &str, initial_input: Option<serde_json::Value>) -> Result<Self, String> {
        let constraints = spec.constraints.clone().unwrap_or(AgentConstraints {
            max_steps: 10,
            timeout_seconds: 180,
            token_budget_per_run: 15000,
            requires_human_approval: false,
        });

        let circuit_breaker = CircuitBreaker::new(
            constraints.max_steps,
            constraints.timeout_seconds,
            constraints.token_budget_per_run,
        );

        let mut modules_map = HashMap::new();
        for m in &spec.modules {
            modules_map.insert(m.id.clone(), m.clone());
        }

        let mut state_data = HashMap::new();
        if let Some(input) = initial_input {
            state_data.insert("initial_input".to_string(), input);
        }

        let context = WorkerContext {
            run_id: run_id.to_string(),
            agent_id: spec.id.clone(),
            state_data,
            step_logs: Vec::new(),
        };

        Ok(Self {
            spec,
            context,
            circuit_breaker,
            modules_map,
        })
    }

    /// Event loop eksekusi transisi state simpul ke simpul berikutnya
    pub fn execute(&mut self) -> WorkerExecutionReport {
        let global_start = Instant::now();

        // 1. Validasi integritas topologi DAG dan tentukan urutan linear deterministik
        let val_res = validate_and_sort_dag(&self.spec.pipeline_dag);
        if !val_res.is_valid {
            return WorkerExecutionReport {
                run_id: self.context.run_id.clone(),
                agent_id: self.spec.id.clone(),
                status: "FAILED".to_string(),
                execution_order: Vec::new(),
                steps_executed: 0,
                total_duration_us: global_start.elapsed().as_micros(),
                total_tokens_consumed: 0,
                output_summary: "Validasi DAG gagal".to_string(),
                step_logs: Vec::new(),
                final_state: self.context.state_data.clone(),
                error: Some(val_res.errors.join("; ")),
            };
        }

        let execution_order = val_res.execution_order;
        let mut total_tokens = 0u64;

        // 2. Loop transisi state antar-simpul (Event Loop Transisi State)
        for node_id in &execution_order {
            // Evaluasi batas sirkuit (Circuit Breaker check)
            if let Err(trip_err) = self.circuit_breaker.check_before_step(node_id) {
                return WorkerExecutionReport {
                    run_id: self.context.run_id.clone(),
                    agent_id: self.spec.id.clone(),
                    status: "CIRCUIT_BROKEN".to_string(),
                    execution_order: execution_order.clone(),
                    steps_executed: self.context.step_logs.len(),
                    total_duration_us: global_start.elapsed().as_micros(),
                    total_tokens_consumed: total_tokens,
                    output_summary: format!("Eksekusi dihentikan oleh sirkuit pelindung: {trip_err}"),
                    step_logs: self.context.step_logs.clone(),
                    final_state: self.context.state_data.clone(),
                    error: Some(trip_err),
                };
            }

            let step_start = Instant::now();
            let module_spec = self.modules_map.get(node_id).cloned().unwrap_or_else(|| ModuleSpec {
                id: node_id.clone(),
                module_type: "generic".to_string(),
                provider: "internal".to_string(),
                name: Some(node_id.clone()),
                params: json!({}),
            });

            // Jalankan logika evaluasi modul secara deterministik
            let (step_output, tokens_consumed) = self.dispatch_module_eval(&module_spec);
            let step_duration_us = step_start.elapsed().as_micros();
            total_tokens += tokens_consumed;

            // Catat ke state data
            let out_snippet = serde_json::to_string(&step_output).unwrap_or_default();
            self.context.state_data.insert(format!("node_{node_id}_output"), step_output);

            let log_entry = StepLog {
                node_id: node_id.clone(),
                module_type: module_spec.module_type.clone(),
                provider: module_spec.provider.clone(),
                status: "SUCCESS".to_string(),
                duration_us: step_duration_us,
                output_snippet: out_snippet.chars().take(120).collect(),
                tokens_consumed,
            };
            self.context.step_logs.push(log_entry);

            // Periksa loop berulang / thrashing
            if let Err(loop_err) = self.circuit_breaker.record_step_outcome(node_id, &out_snippet, tokens_consumed) {
                return WorkerExecutionReport {
                    run_id: self.context.run_id.clone(),
                    agent_id: self.spec.id.clone(),
                    status: "CIRCUIT_BROKEN".to_string(),
                    execution_order: execution_order.clone(),
                    steps_executed: self.context.step_logs.len(),
                    total_duration_us: global_start.elapsed().as_micros(),
                    total_tokens_consumed: total_tokens,
                    output_summary: format!("Eksekusi diputus karena loop berulang: {loop_err}"),
                    step_logs: self.context.step_logs.clone(),
                    final_state: self.context.state_data.clone(),
                    error: Some(loop_err),
                };
            }
        }

        let total_duration_us = global_start.elapsed().as_micros();
        let steps_executed = self.context.step_logs.len();
        let summary = format!(
            "Worker ephemeral menyelesaikan {} simpul DAG dalam {}us ({}ms) dengan konsumsi {} token.",
            steps_executed,
            total_duration_us,
            total_duration_us / 1000,
            total_tokens
        );

        WorkerExecutionReport {
            run_id: self.context.run_id.clone(),
            agent_id: self.spec.id.clone(),
            status: "COMPLETED".to_string(),
            execution_order,
            steps_executed,
            total_duration_us,
            total_tokens_consumed: total_tokens,
            output_summary: summary,
            step_logs: self.context.step_logs.clone(),
            final_state: self.context.state_data.clone(),
            error: None,
        }
    }

    /// Evaluator transisi state internal per modul
    fn dispatch_module_eval(&self, module: &ModuleSpec) -> (serde_json::Value, u64) {
        match module.module_type.as_str() {
            "trigger" => {
                let out = json!({
                    "event": "DISPATCH_TRIGGERED",
                    "provider": module.provider,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "state": "ACTIVE"
                });
                (out, 0)
            }
            "context" => {
                let out = json!({
                    "context_loaded": true,
                    "provider": module.provider,
                    "params": module.params,
                    "entries_count": 1
                });
                (out, 0)
            }
            "reasoning" => {
                let out = json!({
                    "thought": format!("Mengevaluasi keputusan pada simpul '{}'", module.id),
                    "action": "ROUTE_TO_NEXT_NODE",
                    "confidence": 0.98,
                    "provider": module.provider
                });
                (out, 45) // Konsumsi nominal token reasoning
            }
            "tool" => {
                let out = json!({
                    "tool_executed": module.id,
                    "provider": module.provider,
                    "result": "OK",
                    "payload": module.params
                });
                (out, 12)
            }
            "guardrail" => {
                let out = json!({
                    "guardrail_passed": true,
                    "inspector": module.provider,
                    "violations": []
                });
                (out, 5)
            }
            "output" => {
                let out = json!({
                    "status": "DELIVERED",
                    "channel": module.provider,
                    "delivered_at": chrono::Utc::now().to_rfc3339()
                });
                (out, 0)
            }
            _ => {
                let out = json!({
                    "status": "PASSED",
                    "node_id": module.id,
                    "provider": module.provider
                });
                (out, 0)
            }
        }
    }
}

/// Helper untuk spawn ephemeral worker di task terisolasi (async spawn)
pub async fn spawn_ephemeral_worker(
    spec: AgentSpec,
    run_id: String,
    initial_input: Option<serde_json::Value>,
) -> Result<WorkerExecutionReport, String> {
    tokio::task::spawn_blocking(move || {
        let mut worker = EphemeralWorker::new(spec, &run_id, initial_input)?;
        Ok(worker.execute())
    })
    .await
    .map_err(|e| format!("Join error pada worker thread: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DagEdge, PipelineDag};

    #[tokio::test]
    async fn test_ephemeral_worker_execution() {
        let spec = AgentSpec {
            id: "test-agent".to_string(),
            name: "Test Agent Engine".to_string(),
            version: Some("1.0.0".to_string()),
            category: Some("TEST".to_string()),
            description: Some("Uji worker ephemeral".to_string()),
            author: Some("Bagas Cihuy".to_string()),
            constraints: Some(AgentConstraints {
                max_steps: 5,
                timeout_seconds: 10,
                token_budget_per_run: 5000,
                requires_human_approval: false,
            }),
            trigger: None,
            modules: vec![
                ModuleSpec {
                    id: "trigger_node".to_string(),
                    module_type: "trigger".to_string(),
                    provider: "manual".to_string(),
                    name: Some("Trigger".to_string()),
                    params: json!({}),
                },
                ModuleSpec {
                    id: "reasoning_node".to_string(),
                    module_type: "reasoning".to_string(),
                    provider: "9router".to_string(),
                    name: Some("Reasoning".to_string()),
                    params: json!({}),
                },
                ModuleSpec {
                    id: "output_node".to_string(),
                    module_type: "output".to_string(),
                    provider: "stdout".to_string(),
                    name: Some("Output".to_string()),
                    params: json!({}),
                },
            ],
            pipeline_dag: PipelineDag {
                nodes: vec![
                    "trigger_node".to_string(),
                    "reasoning_node".to_string(),
                    "output_node".to_string(),
                ],
                edges: vec![
                    DagEdge {
                        from: "trigger_node".to_string(),
                        to: "reasoning_node".to_string(),
                    },
                    DagEdge {
                        from: "reasoning_node".to_string(),
                        to: "output_node".to_string(),
                    },
                ],
            },
        };

        let report = spawn_ephemeral_worker(spec, "run-test-1".to_string(), None)
            .await
            .expect("Worker harus berhasil dieksekusi");

        assert_eq!(report.status, "COMPLETED");
        assert_eq!(report.steps_executed, 3);
        assert_eq!(report.execution_order, vec!["trigger_node", "reasoning_node", "output_node"]);
        assert!(report.total_duration_us > 0);
        assert!(report.total_duration_us < 10_000, "Eksekusi ephemeral harus sub-10ms (cold-start cepat)");
    }
}
