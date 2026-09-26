use crate::dag;
use crate::model::{AgentConstraints, AgentSpec, DagEdge, ModuleSpec, PipelineDag, TriggerConfig};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCompilerRequest {
    pub prompt: String,
    pub name: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCompilerResponse {
    pub success: bool,
    pub agent: AgentSpec,
    pub execution_order: Vec<String>,
    pub detected_intent: DetectedIntent,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedIntent {
    pub trigger_type: String,
    pub has_research: bool,
    pub has_reasoning: bool,
    pub has_action: bool,
    pub has_storage: bool,
    pub primary_domain: String,
}

/// Heuristik deterministik mengompilasi deskripsi bahasa alami menjadi struktur AgentSpec dan PipelineDag yang valid.
pub fn compile_prompt_to_agent(req: &PromptCompilerRequest) -> Result<PromptCompilerResponse, String> {
    let raw_prompt = req.prompt.trim();
    if raw_prompt.is_empty() {
        return Err("Prompt deskripsi agen tidak boleh kosong".to_string());
    }

    let p_lower = raw_prompt.to_lowercase();

    // 1. Ekstraksi Niat & Intent
    let mut trigger_type = "webhook".to_string();
    if p_lower.contains("setiap")
        || p_lower.contains("tiap")
        || p_lower.contains("jadwal")
        || p_lower.contains("cron")
        || p_lower.contains("menit")
        || p_lower.contains("jam")
        || p_lower.contains("daily")
        || p_lower.contains("rutin")
    {
        trigger_type = "cron".to_string();
    }

    let has_research = p_lower.contains("cari")
        || p_lower.contains("search")
        || p_lower.contains("riset")
        || p_lower.contains("baca")
        || p_lower.contains("pantau")
        || p_lower.contains("ambil")
        || p_lower.contains("fetch")
        || p_lower.contains("url")
        || p_lower.contains("scrape")
        || p_lower.contains("web");

    let has_reasoning = p_lower.contains("analisis")
        || p_lower.contains("ringkas")
        || p_lower.contains("evaluasi")
        || p_lower.contains("rangkum")
        || p_lower.contains("audit")
        || p_lower.contains("review")
        || p_lower.contains("pikir")
        || p_lower.contains("tinjau")
        || p_lower.contains("klasifikasi")
        || p_lower.contains("sarikan")
        || (!has_research && !p_lower.contains("simpan"));

    let has_action = p_lower.contains("eksekusi")
        || p_lower.contains("jalankan")
        || p_lower.contains("kirim")
        || p_lower.contains("post")
        || p_lower.contains("notifikasi")
        || p_lower.contains("telegram")
        || p_lower.contains("webhook")
        || p_lower.contains("terminal")
        || p_lower.contains("bash")
        || p_lower.contains("lapor");

    let has_storage = p_lower.contains("simpan")
        || p_lower.contains("catat")
        || p_lower.contains("vault")
        || p_lower.contains("obsidian")
        || p_lower.contains("buku")
        || p_lower.contains("database")
        || p_lower.contains("kv")
        || p_lower.contains("rekam")
        || p_lower.contains("arsip");

    // Domain klasifikasi
    let primary_domain = if p_lower.contains("pasar")
        || p_lower.contains("harga")
        || p_lower.contains("trading")
        || p_lower.contains("crypto")
        || p_lower.contains("saham")
    {
        "FINANCE".to_string()
    } else if p_lower.contains("kode")
        || p_lower.contains("git")
        || p_lower.contains("pr")
        || p_lower.contains("review")
        || p_lower.contains("bug")
        || p_lower.contains("rust")
    {
        "DEV".to_string()
    } else if p_lower.contains("server")
        || p_lower.contains("ram")
        || p_lower.contains("cpu")
        || p_lower.contains("sentinel")
        || p_lower.contains("health")
    {
        "OPS".to_string()
    } else if p_lower.contains("konten")
        || p_lower.contains("sosial")
        || p_lower.contains("post")
        || p_lower.contains("media")
        || p_lower.contains("warta")
    {
        "GROWTH".to_string()
    } else {
        "GENERAL".to_string()
    };

    let detected_intent = DetectedIntent {
        trigger_type: trigger_type.clone(),
        has_research,
        has_reasoning,
        has_action,
        has_storage,
        primary_domain: primary_domain.clone(),
    };

    // 2. Tentukan ID & Nama Agen
    let slug_base = raw_prompt
        .split_whitespace()
        .take(4)
        .collect::<Vec<&str>>()
        .join("-")
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let agent_id = format!(
        "agent-auto-{}",
        if slug_base.is_empty() {
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        } else {
            slug_base
        }
    );

    let agent_name = req.name.clone().unwrap_or_else(|| {
        let first_sentence = raw_prompt.split('.').next().unwrap_or(raw_prompt);
        let trimmed = first_sentence.trim();
        if trimmed.len() > 40 {
            format!("{}...", &trimmed[..37])
        } else {
            trimmed.to_string()
        }
    });

    let category = req.category.clone().unwrap_or(primary_domain);

    // 3. Merakit Modules dan Bus Relasi DAG
    let mut modules: Vec<ModuleSpec> = Vec::new();
    let mut node_ids: Vec<String> = Vec::new();
    let mut edges: Vec<DagEdge> = Vec::new();

    let mut previous_node_id: Option<String> = None;

    // Module 1 (Opsional): Research / Context Reader / Web Search
    if has_research {
        let res_id = "step_search".to_string();
        modules.push(ModuleSpec {
            id: res_id.clone(),
            module_type: "tool".to_string(),
            provider: "builtin.web_search".to_string(),
            name: Some("Penyerap Fakta Web & Konteks".to_string()),
            params: json!({
                "query": raw_prompt,
                "limit": 5
            }),
        });
        node_ids.push(res_id.clone());
        previous_node_id = Some(res_id);
    }

    // Module 2: Core Reasoning (ReAct / LLM)
    let reason_id = "step_reasoning".to_string();
    let reason_input_binding = if let Some(prev) = &previous_node_id {
        format!("{{{{{}.output}}}}", prev)
    } else {
        "{{input}}".to_string()
    };

    modules.push(ModuleSpec {
        id: reason_id.clone(),
        module_type: "reasoning".to_string(),
        provider: "hermes.reasoning.react".to_string(),
        name: Some("Penalaran Kognitif & Sintesis".to_string()),
        params: json!({
            "model": "ag/claude-sonnet-4-6",
            "temperature": 0.2,
            "system_prompt": format!("Kamu adalah agen otonom khusus {}. Jalankan tugas: '{}' secara presisi, zero-slop, dan to the point.", agent_name, raw_prompt),
            "input_binding": reason_input_binding
        }),
    });
    node_ids.push(reason_id.clone());

    if let Some(prev) = previous_node_id {
        edges.push(DagEdge {
            from: prev,
            to: reason_id.clone(),
        });
    }
    previous_node_id = Some(reason_id);

    // Module 3 (Opsional): Action / Execution / Dispatcher
    if has_action {
        let action_id = "step_action".to_string();
        modules.push(ModuleSpec {
            id: action_id.clone(),
            module_type: "tool".to_string(),
            provider: "builtin.terminal_exec".to_string(),
            name: Some("Dispatser Aksi Lapangan".to_string()),
            params: json!({
                "command": "echo 'Agent action dispatched successfully'",
                "timeout_seconds": 15
            }),
        });
        node_ids.push(action_id.clone());

        if let Some(prev) = previous_node_id {
            edges.push(DagEdge {
                from: prev,
                to: action_id.clone(),
            });
        }
        previous_node_id = Some(action_id);
    }

    // Module 4 (Opsional): Storage / Output Sink
    if has_storage || !has_action {
        let store_id = "step_sink".to_string();
        modules.push(ModuleSpec {
            id: store_id.clone(),
            module_type: "output".to_string(),
            provider: "builtin.vault_reader".to_string(),
            name: Some("Pencatat Obsidian Vault".to_string()),
            params: json!({
                "vault_path": "/home/ubuntu/otak-koding/BUKU_CATATAN",
                "format": "markdown",
                "category": category.clone()
            }),
        });
        node_ids.push(store_id.clone());

        if let Some(prev) = previous_node_id {
            edges.push(DagEdge {
                from: prev,
                to: store_id,
            });
        }
    }

    let dag = PipelineDag {
        nodes: node_ids,
        edges,
    };

    // 4. Validasi Topologi DAG Secara Algoritmik
    let validation = dag::validate_and_sort_dag(&dag);
    if !validation.is_valid {
        return Err(format!(
            "Hasil kompilasi topologi DAG menghasilkan inkonsistensi: {:?}",
            validation.errors
        ));
    }

    let trigger_config = if trigger_type == "cron" {
        TriggerConfig {
            trigger_type: "cron".to_string(),
            schedule: Some("0 */30 * * * *".to_string()),
            timezone: Some("Asia/Jakarta".to_string()),
            path: None,
        }
    } else {
        TriggerConfig {
            trigger_type: "webhook".to_string(),
            schedule: None,
            timezone: None,
            path: Some(format!("/webhook/{}", agent_id)),
        }
    };

    let agent_spec = AgentSpec {
        id: agent_id,
        name: agent_name,
        version: Some("1.0.0".to_string()),
        category: Some(category),
        description: Some(format!("Hasil kompilasi prompt otonom: {}", raw_prompt)),
        author: Some("Bagas Cihuy".to_string()),
        constraints: Some(AgentConstraints {
            max_steps: 6,
            timeout_seconds: 120,
            token_budget_per_run: 8000,
            requires_human_approval: false,
        }),
        trigger: Some(trigger_config),
        modules,
        pipeline_dag: dag,
    };

    let explanation = format!(
        "Agen berhasil dikompilasi secara deterministik dari prompt pengguna. Terdiri dari {} simpul yang terhubung dalam topologi Directed Acyclic Graph bebas siklus (Order: {}).",
        agent_spec.pipeline_dag.nodes.len(),
        validation.execution_order.join(" -> ")
    );

    Ok(PromptCompilerResponse {
        success: true,
        agent: agent_spec,
        execution_order: validation.execution_order,
        detected_intent,
        explanation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_prompt_finance_monitor() {
        let req = PromptCompilerRequest {
            prompt: "Setiap 15 menit pantau harga pasar BTC dan lakukan analisis tren lalu simpan ke vault".to_string(),
            name: Some("Bitcoin Market Sentinel".to_string()),
            category: None,
        };

        let res = compile_prompt_to_agent(&req).expect("Kompilasi prompt gagal");
        assert!(res.success);
        assert_eq!(res.detected_intent.trigger_type, "cron");
        assert!(res.detected_intent.has_research);
        assert!(res.detected_intent.has_reasoning);
        assert!(res.detected_intent.has_storage);
        assert_eq!(res.detected_intent.primary_domain, "FINANCE");
        assert!(!res.execution_order.is_empty());
    }

    #[test]
    fn test_compile_prompt_empty_fails() {
        let req = PromptCompilerRequest {
            prompt: "   ".to_string(),
            name: None,
            category: None,
        };
        let res = compile_prompt_to_agent(&req);
        assert!(res.is_err());
    }

    #[test]
    fn test_compile_prompt_webhook_code_review() {
        let req = PromptCompilerRequest {
            prompt: "Terima payload commit git baru, tinjau kualitas kode rust dan kirim laporannya".to_string(),
            name: None,
            category: None,
        };

        let res = compile_prompt_to_agent(&req).expect("Kompilasi webhook gagal");
        assert!(res.success);
        assert_eq!(res.detected_intent.trigger_type, "webhook");
        assert!(res.detected_intent.has_reasoning);
        assert!(res.detected_intent.has_action);
        assert_eq!(res.detected_intent.primary_domain, "DEV");
    }
}
