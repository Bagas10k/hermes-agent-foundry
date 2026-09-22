use rusqlite::{params, Connection, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub file_path: String,
    pub symbol_name: String,
    pub symbol_kind: String, // "fn", "struct", "enum", "trait", "route"
    pub signature: String,
    pub line_number: usize,
}

#[allow(dead_code)]
pub struct CodeIntelEngine {
    conn: Connection,
}

#[allow(dead_code)]
impl CodeIntelEngine {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS file_hashes (
                 file_path TEXT PRIMARY KEY,
                 sha256 TEXT NOT NULL,
                 last_indexed_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS code_symbols (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 file_path TEXT NOT NULL,
                 symbol_name TEXT NOT NULL,
                 symbol_kind TEXT NOT NULL,
                 signature TEXT NOT NULL,
                 line_number INTEGER NOT NULL,
                 FOREIGN KEY(file_path) REFERENCES file_hashes(file_path) ON DELETE CASCADE
             );
             CREATE INDEX IF NOT EXISTS idx_symbol_name ON code_symbols(symbol_name);
             CREATE INDEX IF NOT EXISTS idx_symbol_file ON code_symbols(file_path);",
        )?;
        Ok(Self { conn })
    }

    /// Menghitung hash SHA-256 berkas untuk deteksi perubahan inkremental
    pub fn calculate_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Memeriksa apakah berkas sudah berubah sejak indeks terakhir.
    /// Mengembalikan true jika berkas berubah atau belum pernah diindeks.
    pub fn is_file_stale(&self, file_path: &str, current_hash: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT sha256 FROM file_hashes WHERE file_path = ?1",
        )?;
        let mut rows = stmt.query(params![file_path])?;
        if let Some(row) = rows.next()? {
            let stored_hash: String = row.get(0)?;
            Ok(stored_hash != current_hash)
        } else {
            Ok(true) // Belum pernah diindeks
        }
    }

    /// Mengindeks struktur berkas secara selektif jika terjadi perubahan hash
    pub fn index_file_if_changed(&mut self, file_path: &str) -> Result<bool> {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Ok(false),
        };

        let current_hash = Self::calculate_hash(&content);
        if !self.is_file_stale(file_path, &current_hash)? {
            // Hash identik: TIDAK PERLU ANALISIS ULANG (Hemat token & komputasi)
            return Ok(false);
        }

        let now = chrono::Utc::now().timestamp();
        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT INTO file_hashes (file_path, sha256, last_indexed_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(file_path) DO UPDATE SET sha256 = ?2, last_indexed_at = ?3",
            params![file_path, current_hash, now],
        )?;

        tx.execute(
            "DELETE FROM code_symbols WHERE file_path = ?1",
            params![file_path],
        )?;

        // Ekstraksi signature simbolis ringan (fn, struct, enum, pub async fn)
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let line_num = idx + 1;

            if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") || trimmed.starts_with("pub async fn ") {
                let name = extract_symbol_name(trimmed);
                tx.execute(
                    "INSERT INTO code_symbols (file_path, symbol_name, symbol_kind, signature, line_number)
                     VALUES (?1, ?2, 'fn', ?3, ?4)",
                    params![file_path, name, trimmed, line_num],
                )?;
            } else if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                let name = extract_symbol_name(trimmed);
                tx.execute(
                    "INSERT INTO code_symbols (file_path, symbol_name, symbol_kind, signature, line_number)
                     VALUES (?1, ?2, 'struct', ?3, ?4)",
                    params![file_path, name, trimmed, line_num],
                )?;
            } else if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
                let name = extract_symbol_name(trimmed);
                tx.execute(
                    "INSERT INTO code_symbols (file_path, symbol_name, symbol_kind, signature, line_number)
                     VALUES (?1, ?2, 'enum', ?3, ?4)",
                    params![file_path, name, trimmed, line_num],
                )?;
            }
        }

        tx.commit()?;
        Ok(true)
    }

    /// Menghasilkan Skeleton Ringkas (Hanya signature struktural) tanpa memuat ribuan baris kode
    pub fn get_structural_skeleton(&self, file_path: &str) -> Result<String> {
        let mut stmt = self.conn.prepare(
            "SELECT symbol_kind, symbol_name, signature, line_number
             FROM code_symbols
             WHERE file_path = ?1
             ORDER BY line_number ASC",
        )?;
        let rows = stmt.query_map(params![file_path], |row| {
            let kind: String = row.get(0)?;
            let name: String = row.get(1)?;
            let sig: String = row.get(2)?;
            let line: usize = row.get(3)?;
            Ok(format!("L{:04} [{}] {} -> {}", line, kind.to_uppercase(), name, sig))
        })?;

        let mut skeleton = Vec::new();
        for r in rows {
            skeleton.push(r?);
        }
        Ok(skeleton.join("\n"))
    }
}

#[allow(dead_code)]
fn extract_symbol_name(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, p) in parts.iter().enumerate() {
        if (*p == "fn" || *p == "struct" || *p == "enum") && i + 1 < parts.len() {
            let raw_name = parts[i + 1];
            return raw_name.split('(').next().unwrap_or(raw_name).split('<').next().unwrap_or(raw_name).replace('{', "").to_string();
        }
    }
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_intel_incremental_indexing() {
        let mut engine = CodeIntelEngine::new(":memory:").unwrap();
        let hash1 = CodeIntelEngine::calculate_hash("pub fn test_run() {}");
        assert_eq!(hash1.len(), 64);
        assert!(engine.is_file_stale("src/dummy.rs", &hash1).unwrap());
    }
}
