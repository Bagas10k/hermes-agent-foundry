use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueFinding {
    pub rule_id: String,
    pub severity: String, // "CRITICAL", "WARNING", "INFO"
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueVerdict {
    pub passed: bool,
    pub score: u8, // 0 - 100
    pub findings: Vec<CritiqueFinding>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Default)]
pub struct CriticGuardrail {
    pub disallow_empty_output: bool,
    pub disallow_raw_secrets: bool,
    pub disallow_hallucinated_urls: bool,
    pub min_output_length: usize,
    pub max_output_length: usize,
}

impl CriticGuardrail {
    pub fn standard() -> Self {
        Self {
            disallow_empty_output: true,
            disallow_raw_secrets: true,
            disallow_hallucinated_urls: true,
            min_output_length: 5,
            max_output_length: 50_000,
        }
    }

    pub fn audit_candidate_action(&self, action: &str, input: &str) -> CritiqueVerdict {
        let mut findings = Vec::new();
        let mut score: u8 = 100;

        let trimmed_input = input.trim();
        if trimmed_input.is_empty() && action != "FinalAnswer" && action != "finish" {
            findings.push(CritiqueFinding {
                rule_id: "EMPTY_ACTION_INPUT".to_string(),
                severity: "WARNING".to_string(),
                message: format!("Aksi '{action}' dijalankan dengan input kosong"),
            });
            score = score.saturating_sub(20);
        }

        let dangerous_patterns = ["rm -rf", "mkfs", ":(){ :|:& };:", "drop database", "truncate table"];
        for pat in dangerous_patterns {
            if trimmed_input.to_lowercase().contains(pat) {
                findings.push(CritiqueFinding {
                    rule_id: "DESTRUCTIVE_COMMAND_DETECTED".to_string(),
                    severity: "CRITICAL".to_string(),
                    message: format!("Pola destruktif berbahaya terdeteksi: '{pat}'"),
                });
                score = 0;
                break;
            }
        }

        let passed = score >= 70 && !findings.iter().any(|f| f.severity == "CRITICAL");
        let recommendation = if passed {
            "Tindakan disetujui untuk dieksekusi".to_string()
        } else {
            "Tindakan ditolak oleh Critic Guardrail: tinjau temuan kritis".to_string()
        };

        CritiqueVerdict {
            passed,
            score,
            findings,
            recommendation,
        }
    }

    pub fn audit_final_output(&self, output: &str) -> CritiqueVerdict {
        let mut findings = Vec::new();
        let mut score: u8 = 100;

        let trimmed = output.trim();
        if self.disallow_empty_output && trimmed.is_empty() {
            findings.push(CritiqueFinding {
                rule_id: "EMPTY_FINAL_OUTPUT".to_string(),
                severity: "CRITICAL".to_string(),
                message: "Hasil akhir agen bernilai kosong".to_string(),
            });
            score = 0;
        }

        if trimmed.len() < self.min_output_length {
            findings.push(CritiqueFinding {
                rule_id: "OUTPUT_TOO_SHORT".to_string(),
                severity: "WARNING".to_string(),
                message: format!("Panjang output ({} chars) di bawah batas minimal {}", trimmed.len(), self.min_output_length),
            });
            score = score.saturating_sub(25);
        }

        if self.disallow_raw_secrets {
            let secret_patterns = ["sk-", "ghp_", "bearer ", "token="];
            for pat in secret_patterns {
                if trimmed.to_lowercase().contains(pat) {
                    findings.push(CritiqueFinding {
                        rule_id: "SECRET_LEAKAGE_DETECTED".to_string(),
                        severity: "CRITICAL".to_string(),
                        message: format!("Potensi kebocoran kredensial terdeteksi: '{pat}'"),
                    });
                    score = 0;
                    break;
                }
            }
        }

        let passed = score >= 70 && !findings.iter().any(|f| f.severity == "CRITICAL");
        let recommendation = if passed {
            "Output memenuhi standar integritas".to_string()
        } else {
            "Output gagal melewati gate audit critic".to_string()
        };

        CritiqueVerdict {
            passed,
            score,
            findings,
            recommendation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critic_action_audit() {
        let critic = CriticGuardrail::standard();
        let v1 = critic.audit_candidate_action("web_search", "AI agent framework");
        assert!(v1.passed);
        assert_eq!(v1.score, 100);

        let v2 = critic.audit_candidate_action("terminal_exec", "rm -rf / --no-preserve-root");
        assert!(!v2.passed);
        assert_eq!(v2.score, 0);
    }

    #[test]
    fn test_critic_output_audit() {
        let critic = CriticGuardrail::standard();
        let v1 = critic.audit_final_output("Analisis selesai. Parameter optimal adalah LR=3e-4.");
        assert!(v1.passed);

        let v2 = critic.audit_final_output("Kunci rahasia API: sk-123456abcdefg");
        assert!(!v2.passed);
        assert!(v2.findings.iter().any(|f| f.rule_id == "SECRET_LEAKAGE_DETECTED"));
    }
}
