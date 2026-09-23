mod circuit_breaker;
mod code_intel;
mod dag;
mod db;
mod engine;
mod model;
pub mod modules;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use model::AgentSpec;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

struct AppState {
    db: Mutex<db::Database>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Inisialisasi basis data SQLite
    let db_path = "data/foundry.sqlite";
    std::fs::create_dir_all("data").unwrap();
    let database = db::Database::init(db_path).expect("Gagal inisialisasi basis data SQLite");

    let state = Arc::new(AppState {
        db: Mutex::new(database),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/dag/validate", post(validate_dag_handler))
        .route("/api/agents", get(list_agents_handler).post(create_or_update_agent_handler))
        .route("/api/agents/:id", get(get_agent_handler))
        .route("/api/agents/:id/trigger", post(trigger_agent_handler))
        .route("/api/templates", get(list_templates_handler))
        .route("/api/runs", get(list_runs_handler))
        .fallback_service(ServeDir::new("public"))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3200".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("=== HERMES AGENT FOUNDRY ENGINE (RUST) AKTIF DI http://{} ===", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "engine": "Hermes Agent Foundry (Rust Edition)",
        "version": "1.0.0",
        "status": "HEALTHY",
        "memory_model": "Zero GC / Ephemeral Worker"
    }))
}

async fn validate_dag_handler(Json(dag): Json<model::PipelineDag>) -> impl IntoResponse {
    let result = dag::validate_and_sort_dag(&dag);
    if result.is_valid {
        (StatusCode::OK, Json(json!({
            "success": true,
            "is_valid": true,
            "execution_order": result.execution_order,
            "errors": []
        })))
    } else {
        (StatusCode::BAD_REQUEST, Json(json!({
            "success": false,
            "is_valid": false,
            "execution_order": [],
            "errors": result.errors
        })))
    }
}

async fn create_or_update_agent_handler(
    State(state): State<Arc<AppState>>,
    Json(spec): Json<AgentSpec>,
) -> impl IntoResponse {
    // 1. Validasi DAG terlebih dahulu
    let val_res = dag::validate_and_sort_dag(&spec.pipeline_dag);
    if !val_res.is_valid {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "success": false,
            "error": "Validasi DAG gagal",
            "details": val_res.errors
        })));
    }

    let spec_json = serde_json::to_string(&spec).unwrap();
    let category = spec.category.unwrap_or_else(|| "GENERAL".to_string());

    let db = state.db.lock().await;
    match db.save_agent(&spec.id, &spec.name, &category, &spec_json) {
        Ok(_) => (StatusCode::OK, Json(json!({
            "success": true,
            "agent_id": spec.id,
            "message": "Agen berhasil disimpan dan diverifikasi secara algoritmik",
            "execution_order": val_res.execution_order
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn list_agents_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock().await;
    match db.list_agents() {
        Ok(agents) => {
            let list: Vec<serde_json::Value> = agents
                .into_iter()
                .map(|(id, name, cat, spec)| {
                    let parsed: serde_json::Value = serde_json::from_str(&spec).unwrap_or(json!({}));
                    json!({
                        "id": id,
                        "name": name,
                        "category": cat,
                        "spec": parsed
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!({ "success": true, "agents": list })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))),
    }
}

async fn get_agent_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let db = state.db.lock().await;
    match db.get_agent(&id) {
        Ok(Some(spec)) => {
            let parsed: serde_json::Value = serde_json::from_str(&spec).unwrap_or(json!({}));
            (StatusCode::OK, Json(json!({ "success": true, "agent": parsed })))
        }
        Ok(None) => {
            // Cek template
            if let Ok(entries) = std::fs::read_dir("templates") {
                for entry in entries.flatten() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if val.get("id").and_then(|v| v.as_str()) == Some(&id) {
                                return (StatusCode::OK, Json(json!({ "success": true, "agent": val })));
                            }
                        }
                    }
                }
            }
            (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Agen tidak ditemukan" })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": e.to_string() }))),
    }
}

async fn list_templates_handler() -> impl IntoResponse {
    let mut templates = Vec::new();
    if let Ok(entries) = std::fs::read_dir("templates") {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    templates.push(val);
                }
            }
        }
    }
    Json(json!({ "success": true, "templates": templates }))
}

async fn trigger_agent_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    payload: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    let started_at = chrono::Utc::now().to_rfc3339();
    let run_id = uuid::Uuid::new_v4().to_string();

    let db = state.db.lock().await;
    
    // 1. Ambil spec agen dari DB atau cek apakah ini template bawaan
    let spec_opt = match db.get_agent(&id) {
        Ok(Some(s)) => Some(s),
        _ => {
            let mut found = None;
            if let Ok(entries) = std::fs::read_dir("templates") {
                for entry in entries.flatten() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if val.get("id").and_then(|v| v.as_str()) == Some(&id) {
                                found = Some(content);
                                break;
                            }
                        }
                    }
                }
            }
            found
        }
    };

    let (spec, spec_json) = match spec_opt {
        Some(s) => match serde_json::from_str::<AgentSpec>(&s) {
            Ok(spec_obj) => (spec_obj, s),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "error": format!("Gagal mem-parsing skema agen: {}", e)
                    })),
                );
            }
        },
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "error": format!("Agen dengan ID '{}' tidak ditemukan di database maupun katalog template", id)
                })),
            );
        }
    };

    // Pastikan agen tersimpan di tabel agents jika merupakan template bawaan (agar foreign key valid)
    let category = spec.category.clone().unwrap_or_else(|| "GENERAL".to_string());
    let _ = db.save_agent(&spec.id, &spec.name, &category, &spec_json);

    let input_params = payload.map(|Json(v)| v);

    // 2. Eksekusi melalui Ephemeral Worker Engine (Zero-Allocation Isolated Thread)
    let worker_res = engine::ephemeral_worker::spawn_ephemeral_worker(spec.clone(), run_id.clone(), input_params.clone()).await;

    match worker_res {
        Ok(report) => {
            let completed_at = chrono::Utc::now().to_rfc3339();
            let _ = db.record_run(
                &run_id,
                &spec.id,
                &report.status,
                &started_at,
                &completed_at,
                report.steps_executed,
                report.total_tokens_consumed,
                &report.output_summary,
                report.error.as_deref(),
            );

            (
                StatusCode::OK,
                Json(json!({
                    "success": report.status == "COMPLETED",
                    "run_id": run_id,
                    "agent_id": spec.id,
                    "agent_name": spec.name,
                    "category": spec.category.unwrap_or_else(|| "GENERAL".to_string()),
                    "status": report.status,
                    "execution_order": report.execution_order,
                    "execution_time_us": report.total_duration_us,
                    "execution_time_ms": report.total_duration_us / 1000,
                    "steps_executed": report.steps_executed,
                    "tokens_consumed": report.total_tokens_consumed,
                    "input_received": input_params.unwrap_or(json!({})),
                    "output_summary": report.output_summary,
                    "step_logs": report.step_logs,
                    "final_state": report.final_state,
                    "message": "Eksekusi agen berhasil melalui Ephemeral Worker Engine"
                })),
            )
        }
        Err(err) => {
            let completed_at = chrono::Utc::now().to_rfc3339();
            let _ = db.record_run(
                &run_id,
                &spec.id,
                "FAILED",
                &started_at,
                &completed_at,
                0,
                0,
                "Eksekusi worker gagal",
                Some(&err),
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "run_id": run_id,
                    "agent_id": spec.id,
                    "status": "FAILED",
                    "error": err
                })),
            )
        }
    }
}

async fn list_runs_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock().await;
    match db.list_runs(20) {
        Ok(runs) => {
            let list: Vec<serde_json::Value> = runs
                .into_iter()
                .map(|(id, agent_id, status, started, completed, tokens, summary)| {
                    json!({
                        "id": id,
                        "agent_id": agent_id,
                        "status": status,
                        "started_at": started,
                        "completed_at": completed,
                        "tokens_consumed": tokens,
                        "output_summary": summary
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!({ "success": true, "runs": list })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}
