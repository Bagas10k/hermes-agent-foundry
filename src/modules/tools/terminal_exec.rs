// modules/tools/terminal_exec.rs
// Hermes Agent Foundry :: Sandboxed Terminal Execution Tool Module
// Hak Cipta: Bagas Cihuy (Bagas Saputra)

use std::time::Instant;
use tokio::process::Command;
use crate::modules::tools::ToolCallResult;

pub struct TerminalExecTool {
    timeout_secs: u64,
    blacklist_patterns: Vec<&'static str>,
}

impl Default for TerminalExecTool {
    fn default() -> Self {
        Self {
            timeout_secs: 15,
            blacklist_patterns: vec![
                "rm -rf /",
                "mkfs",
                ":(){ :|:& };:",
                "dd if=",
                "> /dev/sda",
                "> /dev/nvme",
                "chmod -R 777 /",
                "chown -R",
                "shutdown",
                "reboot",
                "init 0",
                "curl -s | bash",
                "wget -O- | bash",
                "sudo rm",
                "drop database",
                "truncate table",
            ],
        }
    }
}

impl TerminalExecTool {
    pub fn new(timeout_secs: u64) -> Self {
        let mut tool = Self::default();
        tool.timeout_secs = timeout_secs;
        tool
    }

    /// Evaluasi keamanan perintah terhadap blacklist destruktif
    pub fn is_command_safe(&self, cmd: &str) -> Result<(), String> {
        let lower = cmd.to_lowercase();
        for &pattern in &self.blacklist_patterns {
            if lower.contains(pattern) {
                return Err(format!(
                    "Perintah ditolak oleh sandboxing guardrail: mendeteksi pola berbahaya '{}'",
                    pattern
                ));
            }
        }
        Ok(())
    }

    /// Eksekusi perintah shell dengan isolasi timeout dan pemangkasan output
    pub async fn execute(&self, cmd: &str) -> ToolCallResult {
        let start = Instant::now();

        if let Err(err_msg) = self.is_command_safe(cmd) {
            return ToolCallResult {
                success: false,
                output: String::new(),
                error: Some(err_msg),
                execution_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        let timeout_duration = std::time::Duration::from_secs(self.timeout_secs);
        let exec_future = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output();

        match tokio::time::timeout(timeout_duration, exec_future).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

                let combined_output = if !stderr.is_empty() && !stdout.is_empty() {
                    format!("{}\n[STDERR]:\n{}", stdout, stderr)
                } else if !stdout.is_empty() {
                    stdout
                } else {
                    stderr
                };

                let execution_time_ms = start.elapsed().as_millis() as u64;
                if output.status.success() {
                    ToolCallResult {
                        success: true,
                        output: combined_output,
                        error: None,
                        execution_time_ms,
                    }
                } else {
                    ToolCallResult {
                        success: false,
                        output: combined_output,
                        error: Some(format!("Perintah keluar dengan kode error {:?}", output.status.code())),
                        execution_time_ms,
                    }
                }
            }
            Ok(Err(e)) => ToolCallResult {
                success: false,
                output: String::new(),
                error: Some(format!("Gagal spawn proses: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
            Err(_) => ToolCallResult {
                success: false,
                output: String::new(),
                error: Some(format!("Eksekusi melebihi batas waktu (timeout {} detik)", self.timeout_secs)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist_destructive_command() {
        let tool = TerminalExecTool::default();
        assert!(tool.is_command_safe("rm -rf / --no-preserve-root").is_err());
        assert!(tool.is_command_safe("shutdown -h now").is_err());
        assert!(tool.is_command_safe("echo hello world").is_ok());
    }

    #[tokio::test]
    async fn test_terminal_exec_safe_command() {
        let tool = TerminalExecTool::default();
        let res = tool.execute("echo 'foundry test'").await;
        assert!(res.success);
        assert_eq!(res.output, "foundry test");
        assert!(res.error.is_none());
    }

    #[tokio::test]
    async fn test_terminal_exec_blocked_command() {
        let tool = TerminalExecTool::default();
        let res = tool.execute("rm -rf /tmp/test && mkfs.ext4 /dev/sda").await;
        assert!(!res.success);
        assert!(res.error.unwrap().contains("pola berbahaya"));
    }
}
