mod circuit_breaker;
mod dag;
mod db;
mod model;

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
        .route("/api/templates", get(list_templates_handler))
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
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Agen tidak ditemukan" }))),
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
