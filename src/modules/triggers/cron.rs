use serde::{Deserialize, Serialize};
use serde_json::json;

/// Konfigurasi pemicu Cron untuk agen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronTriggerConfig {
    pub schedule: String,         // e.g., "0 2 * * *" atau "@hourly"
    pub timezone: Option<String>, // e.g., "Asia/Jakarta" / "UTC"
    pub job_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronTriggerEvent {
    pub fired_at: String,
    pub schedule: String,
    pub timezone: String,
    pub tick_id: String,
    pub status: String,
}

/// Handler pemicu Cron (Bridge ke ticker jadwal internal & hermes cron scheduler)
pub struct CronTriggerHandler;

impl CronTriggerHandler {
    pub fn parse_config(params: &serde_json::Value) -> Result<CronTriggerConfig, String> {
        let schedule = params.get("schedule")
            .and_then(|v| v.as_str())
            .unwrap_or("0 * * * *")
            .to_string();

        let timezone = params.get("timezone")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let job_name = params.get("job_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(CronTriggerConfig {
            schedule,
            timezone,
            job_name,
        })
    }

    pub fn evaluate(config: &CronTriggerConfig) -> (serde_json::Value, u64) {
        let now = chrono::Utc::now();
        let tz = config.timezone.clone().unwrap_or_else(|| "Asia/Jakarta".to_string());
        let tick_id = format!("tick-{}-{}", config.schedule.replace(' ', "_"), now.timestamp());

        let event = CronTriggerEvent {
            fired_at: now.to_rfc3339(),
            schedule: config.schedule.clone(),
            timezone: tz,
            tick_id,
            status: "TRIGGERED".to_string(),
        };

        (json!(event), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_trigger_eval() {
        let config = CronTriggerConfig {
            schedule: "0 2 * * *".to_string(),
            timezone: Some("Asia/Jakarta".to_string()),
            job_name: Some("nightly-foundry-builder".to_string()),
        };

        let (output, tokens) = CronTriggerHandler::evaluate(&config);
        assert_eq!(tokens, 0);
        assert_eq!(output["schedule"], "0 2 * * *");
        assert_eq!(output["timezone"], "Asia/Jakarta");
        assert_eq!(output["status"], "TRIGGERED");
    }
}
