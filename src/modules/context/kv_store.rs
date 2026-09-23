use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

/// Konfigurasi State Persisten Antar-Run (KV Store)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvStoreConfig {
    pub namespace: String,           // e.g., "agent_state", "metrics", "last_seen"
    pub key: String,                 // Kunci data
    pub op: String,                  // "GET", "SET", "DELETE", "INCREMENT"
    pub value: Option<serde_json::Value>,
    pub ttl_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvStoreResult {
    pub namespace: String,
    pub key: String,
    pub op: String,
    pub success: bool,
    pub value: Option<serde_json::Value>,
    pub updated_at: Option<String>,
}

pub struct KvStoreHandler;

impl KvStoreHandler {
    pub fn init_table<P: AsRef<Path>>(db_path: P) -> SqlResult<()> {
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_memory (
                namespace TEXT NOT NULL,
                key TEXT NOT NULL,
                value_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                expires_at TEXT,
                PRIMARY KEY (namespace, key)
            )",
            [],
        )?;
        Ok(())
    }

    pub fn parse_config(params: &serde_json::Value) -> Result<KvStoreConfig, String> {
        let namespace = params.get("namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("global_memory")
            .to_string();

        let key = params.get("key")
            .and_then(|v| v.as_str())
            .unwrap_or("default_key")
            .to_string();

        let op = params.get("op")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_uppercase();

        let value = params.get("value").cloned();
        let ttl_seconds = params.get("ttl_seconds").and_then(|v| v.as_i64());

        Ok(KvStoreConfig {
            namespace,
            key,
            op,
            value,
            ttl_seconds,
        })
    }

    pub fn execute_op<P: AsRef<Path>>(db_path: P, config: &KvStoreConfig) -> (serde_json::Value, u64) {
        let conn = match Connection::open(db_path) {
            Ok(c) => c,
            Err(e) => {
                return (
                    json!({
                        "error": format!("Gagal membuka SQLite: {e}"),
                        "success": false
                    }),
                    0,
                );
            }
        };

        let now = chrono::Utc::now();
        let now_str = now.to_rfc3339();
        let expires_str = config.ttl_seconds.map(|s| (now + chrono::Duration::seconds(s)).to_rfc3339());

        match config.op.as_str() {
            "GET" => {
                let mut stmt = match conn.prepare("SELECT value_json, updated_at, expires_at FROM kv_memory WHERE namespace = ?1 AND key = ?2") {
                    Ok(s) => s,
                    Err(e) => return (json!({ "error": e.to_string(), "success": false }), 0),
                };

                let mut rows = match stmt.query(params![config.namespace, config.key]) {
                    Ok(r) => r,
                    Err(e) => return (json!({ "error": e.to_string(), "success": false }), 0),
                };

                if let Ok(Some(row)) = rows.next() {
                    let val_str: String = row.get(0).unwrap_or_default();
                    let updated_at: String = row.get(1).unwrap_or_default();
                    let exp: Option<String> = row.get(2).unwrap_or(None);

                    // Cek expired
                    if let Some(exp_at) = exp {
                        if exp_at < now_str {
                            // Hapus expired
                            let _ = conn.execute("DELETE FROM kv_memory WHERE namespace = ?1 AND key = ?2", params![config.namespace, config.key]);
                            return (
                                json!(KvStoreResult {
                                    namespace: config.namespace.clone(),
                                    key: config.key.clone(),
                                    op: "GET".to_string(),
                                    success: true,
                                    value: None,
                                    updated_at: None,
                                }),
                                0,
                            );
                        }
                    }

                    let parsed_val: serde_json::Value = serde_json::from_str(&val_str).unwrap_or(serde_json::Value::String(val_str));
                    (
                        json!(KvStoreResult {
                            namespace: config.namespace.clone(),
                            key: config.key.clone(),
                            op: "GET".to_string(),
                            success: true,
                            value: Some(parsed_val),
                            updated_at: Some(updated_at),
                        }),
                        0,
                    )
                } else {
                    (
                        json!(KvStoreResult {
                            namespace: config.namespace.clone(),
                            key: config.key.clone(),
                            op: "GET".to_string(),
                            success: true,
                            value: None,
                            updated_at: None,
                        }),
                        0,
                    )
                }
            }
            "SET" => {
                let val_json = serde_json::to_string(&config.value.clone().unwrap_or(serde_json::Value::Null)).unwrap_or_default();
                let res = conn.execute(
                    "INSERT INTO kv_memory (namespace, key, value_json, updated_at, expires_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(namespace, key) DO UPDATE SET
                        value_json = excluded.value_json,
                        updated_at = excluded.updated_at,
                        expires_at = excluded.expires_at",
                    params![config.namespace, config.key, val_json, now_str, expires_str],
                );

                match res {
                    Ok(_) => (
                        json!(KvStoreResult {
                            namespace: config.namespace.clone(),
                            key: config.key.clone(),
                            op: "SET".to_string(),
                            success: true,
                            value: config.value.clone(),
                            updated_at: Some(now_str),
                        }),
                        0,
                    ),
                    Err(e) => (json!({ "error": e.to_string(), "success": false }), 0),
                }
            }
            "DELETE" => {
                let res = conn.execute(
                    "DELETE FROM kv_memory WHERE namespace = ?1 AND key = ?2",
                    params![config.namespace, config.key],
                );
                match res {
                    Ok(_) => (
                        json!(KvStoreResult {
                            namespace: config.namespace.clone(),
                            key: config.key.clone(),
                            op: "DELETE".to_string(),
                            success: true,
                            value: None,
                            updated_at: Some(now_str),
                        }),
                        0,
                    ),
                    Err(e) => (json!({ "error": e.to_string(), "success": false }), 0),
                }
            }
            "INCREMENT" => {
                // Ambil nilai lama
                let cur_val: i64 = conn.query_row(
                    "SELECT value_json FROM kv_memory WHERE namespace = ?1 AND key = ?2",
                    params![config.namespace, config.key],
                    |row| {
                        let s: String = row.get(0)?;
                        Ok(s.parse::<i64>().unwrap_or(0))
                    },
                ).unwrap_or(0);

                let delta = config.value.as_ref().and_then(|v| v.as_i64()).unwrap_or(1);
                let new_val = cur_val + delta;

                let _ = conn.execute(
                    "INSERT INTO kv_memory (namespace, key, value_json, updated_at, expires_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(namespace, key) DO UPDATE SET
                        value_json = excluded.value_json,
                        updated_at = excluded.updated_at,
                        expires_at = excluded.expires_at",
                    params![config.namespace, config.key, new_val.to_string(), now_str, expires_str],
                );

                (
                    json!(KvStoreResult {
                        namespace: config.namespace.clone(),
                        key: config.key.clone(),
                        op: "INCREMENT".to_string(),
                        success: true,
                        value: Some(json!(new_val)),
                        updated_at: Some(now_str),
                    }),
                    0,
                )
            }
            _ => (json!({ "error": format!("Operasi tidak didukung: {}", config.op), "success": false }), 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_store_operations() {
        let temp_file = std::env::temp_dir().join("foundry_kv_test.sqlite");
        let _ = std::fs::remove_file(&temp_file);

        KvStoreHandler::init_table(&temp_file).unwrap();

        // 1. SET
        let set_cfg = KvStoreConfig {
            namespace: "agent_alpha".to_string(),
            key: "last_seen_state".to_string(),
            op: "SET".to_string(),
            value: Some(json!({ "status": "ONLINE", "score": 98 })),
            ttl_seconds: None,
        };
        let (out_set, _) = KvStoreHandler::execute_op(&temp_file, &set_cfg);
        assert_eq!(out_set["success"], true);

        // 2. GET
        let get_cfg = KvStoreConfig {
            namespace: "agent_alpha".to_string(),
            key: "last_seen_state".to_string(),
            op: "GET".to_string(),
            value: None,
            ttl_seconds: None,
        };
        let (out_get, _) = KvStoreHandler::execute_op(&temp_file, &get_cfg);
        assert_eq!(out_get["success"], true);
        assert_eq!(out_get["value"]["status"], "ONLINE");

        // 3. INCREMENT
        let inc_cfg = KvStoreConfig {
            namespace: "metrics".to_string(),
            key: "run_counter".to_string(),
            op: "INCREMENT".to_string(),
            value: Some(json!(5)),
            ttl_seconds: None,
        };
        let (out_inc, _) = KvStoreHandler::execute_op(&temp_file, &inc_cfg);
        assert_eq!(out_inc["value"], 5);

        let (out_inc2, _) = KvStoreHandler::execute_op(&temp_file, &inc_cfg);
        assert_eq!(out_inc2["value"], 10);
    }
}
