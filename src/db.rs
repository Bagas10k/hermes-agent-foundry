use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Optimasi performa & keandalan: WAL mode dan busy timeout
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                spec_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS execution_runs (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                steps_executed INTEGER DEFAULT 0,
                tokens_consumed INTEGER DEFAULT 0,
                output_summary TEXT,
                error_message TEXT,
                FOREIGN KEY (agent_id) REFERENCES agents (id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn save_agent(&self, id: &str, name: &str, category: &str, spec_json: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO agents (id, name, category, spec_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                category = excluded.category,
                spec_json = excluded.spec_json,
                updated_at = excluded.updated_at",
            params![id, name, category, spec_json, now],
        )?;
        Ok(())
    }

    pub fn list_agents(&self) -> Result<Vec<(String, String, String, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, name, category, spec_json FROM agents ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;

        let mut list = Vec::new();
        for item in rows {
            list.push(item?);
        }
        Ok(list)
    }

    pub fn get_agent(&self, id: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT spec_json FROM agents WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let spec: String = row.get(0)?;
            Ok(Some(spec))
        } else {
            Ok(None)
        }
    }
}
