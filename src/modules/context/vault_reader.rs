use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

/// Konfigurasi Pembaca Vault Terisolasi (Sandboxed Vault Reader)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultReaderConfig {
    pub vault_root: String,            // Path absolut direktori dasar vault (e.g., /home/ubuntu/otak-koding)
    pub relative_path: String,         // Subpath relatif (e.g., KNOWLEDGE/INDEX.md)
    pub max_lines: Option<usize>,      // Batas maksimum baris (default: 500)
    pub strip_frontmatter: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultReadResult {
    pub target_file: String,
    pub exists: bool,
    pub lines_read: usize,
    pub total_chars: usize,
    pub content: String,
    pub frontmatter: Option<serde_json::Value>,
    pub sandboxed: bool,
}

pub struct VaultReaderHandler;

impl VaultReaderHandler {
    pub fn parse_config(params: &serde_json::Value) -> Result<VaultReaderConfig, String> {
        let vault_root = params.get("vault_root")
            .and_then(|v| v.as_str())
            .unwrap_or("/home/ubuntu/otak-koding")
            .to_string();

        let relative_path = params.get("relative_path")
            .and_then(|v| v.as_str())
            .unwrap_or("KNOWLEDGE/INDEX.md")
            .to_string();

        let max_lines = params.get("max_lines")
            .and_then(|v| v.as_u64())
            .map(|u| u as usize);

        let strip_frontmatter = params.get("strip_frontmatter")
            .and_then(|v| v.as_bool());

        Ok(VaultReaderConfig {
            vault_root,
            relative_path,
            max_lines,
            strip_frontmatter,
        })
    }

    /// Isolasi Keamanan Sandboxing: Mencegah Path Traversal keluar dari vault_root
    pub fn resolve_sandboxed_path(vault_root: &str, relative_path: &str) -> Result<PathBuf, String> {
        let root = Path::new(vault_root);
        let normalized_rel = relative_path.trim_start_matches(['/', '\\']);
        
        // Cek path traversal exploit ("..")
        for component in Path::new(normalized_rel).components() {
            if component.as_os_str() == ".." {
                return Err("Pelanggaran Keamanan Sandboxing: Path Traversal ('..') terdeteksi dilarang".to_string());
            }
        }

        let full_path = root.join(normalized_rel);

        // Jika file sudah ada, canonicalize dan verifikasi prefix
        if full_path.exists() {
            let canon_full = full_path.canonicalize().map_err(|e| e.to_string())?;
            let canon_root = root.canonicalize().map_err(|e| e.to_string())?;
            if !canon_full.starts_with(&canon_root) {
                return Err("Pelanggaran Keamanan Sandboxing: Target di luar root vault".to_string());
            }
            Ok(canon_full)
        } else {
            Ok(full_path)
        }
    }

    pub fn evaluate(config: &VaultReaderConfig) -> (serde_json::Value, u64) {
        let target_path = match Self::resolve_sandboxed_path(&config.vault_root, &config.relative_path) {
            Ok(p) => p,
            Err(e) => {
                return (
                    json!({
                        "error": e,
                        "success": false,
                        "target_file": config.relative_path,
                        "sandboxed": true
                    }),
                    0,
                );
            }
        };

        if !target_path.exists() {
            return (
                json!({
                    "error": format!("Berkas tidak ditemukan: {}", target_path.display()),
                    "success": false,
                    "target_file": config.relative_path,
                    "exists": false,
                    "sandboxed": true
                }),
                0,
            );
        }

        match fs::read_to_string(&target_path) {
            Ok(raw_content) => {
                let max_l = config.max_lines.unwrap_or(500);
                let lines: Vec<&str> = raw_content.lines().take(max_l).collect();
                let truncated_content = lines.join("\n");
                let lines_read = lines.len();
                let total_chars = truncated_content.chars().count();

                let res = VaultReadResult {
                    target_file: config.relative_path.clone(),
                    exists: true,
                    lines_read,
                    total_chars,
                    content: truncated_content,
                    frontmatter: None,
                    sandboxed: true,
                };

                (json!(res), 0)
            }
            Err(e) => (
                json!({
                    "error": format!("Gagal membaca berkas: {e}"),
                    "success": false,
                    "target_file": config.relative_path,
                    "sandboxed": true
                }),
                0,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_vault_reader_sandboxing() {
        let temp_dir = std::env::temp_dir().join("foundry_test_vault");
        fs::create_dir_all(&temp_dir).unwrap();

        let sub_file = temp_dir.join("NOTES.md");
        let mut f = fs::File::create(&sub_file).unwrap();
        writeln!(f, "# Catatan Arsitektur\nBaris kedua").unwrap();

        let config = VaultReaderConfig {
            vault_root: temp_dir.to_str().unwrap().to_string(),
            relative_path: "NOTES.md".to_string(),
            max_lines: Some(10),
            strip_frontmatter: Some(false),
        };

        let (out, tokens) = VaultReaderHandler::evaluate(&config);
        assert_eq!(tokens, 0);
        assert_eq!(out["exists"], true);
        assert_eq!(out["sandboxed"], true);
        assert!(out["content"].as_str().unwrap().contains("Catatan Arsitektur"));

        // Uji Path Traversal Block
        let traversal_config = VaultReaderConfig {
            vault_root: temp_dir.to_str().unwrap().to_string(),
            relative_path: "../../../etc/passwd".to_string(),
            max_lines: Some(10),
            strip_frontmatter: Some(false),
        };
        let (out_block, _) = VaultReaderHandler::evaluate(&traversal_config);
        assert!(out_block["error"].as_str().unwrap().contains("Path Traversal"));
    }
}
