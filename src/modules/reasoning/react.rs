use serde::{Deserialize, Serialize};
use crate::circuit_breaker::CircuitBreaker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActStep {
    pub step: usize,
    pub thought: String,
    pub action: Option<String>,
    pub action_input: Option<String>,
    pub observation: Option<String>,
    pub temperature: f32,
    pub tokens_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActSession {
    pub task: String,
    pub max_steps: usize,
    pub base_temperature: f32,
    pub decay_rate: f32,
    pub steps: Vec<ReActStep>,
    pub final_answer: Option<String>,
    pub is_completed: bool,
}

impl ReActSession {
    pub fn new(task: String, max_steps: usize, base_temperature: f32, decay_rate: f32) -> Self {
        Self {
            task,
            max_steps: if max_steps == 0 { 5 } else { max_steps },
            base_temperature: if base_temperature <= 0.0 { 0.7 } else { base_temperature },
            decay_rate: if decay_rate <= 0.0 || decay_rate >= 1.0 { 0.85 } else { decay_rate },
            steps: Vec::new(),
            final_answer: None,
            is_completed: false,
        }
    }

    /// Menghitung suhu dinamis terdegenerasi (Dynamic Temperature Decay):
    /// T_step = max(T_min, T_base * (decay_rate ^ step))
    /// Menjamin eksplorasi awal fleksibel, lalu mengkristal menjadi deterministik di fase akhir.
    pub fn calculate_decayed_temperature(&self, step: usize) -> f32 {
        let t_min = 0.10f32;
        let decayed = self.base_temperature * self.decay_rate.powi(step as i32);
        if decayed < t_min { t_min } else { decayed }
    }

    pub fn execute_step<F>(
        &mut self,
        circuit_breaker: &mut CircuitBreaker,
        tool_runner: F,
        thought: String,
        action: Option<String>,
        action_input: Option<String>,
        estimated_tokens: u64,
    ) -> Result<ReActStep, String>
    where
        F: FnOnce(&str, &str) -> Result<String, String>,
    {
        if self.is_completed {
            return Err("ReAct session sudah selesai".to_string());
        }

        let current_step_idx = self.steps.len() + 1;
        let node_id = format!("react_step_{}", current_step_idx);

        circuit_breaker.check_before_step(&node_id)?;

        let temp = self.calculate_decayed_temperature(current_step_idx);

        let observation = if let Some(ref act) = action {
            if act == "FinalAnswer" || act == "finish" {
                let answer = action_input.clone().unwrap_or_else(|| thought.clone());
                self.final_answer = Some(answer.clone());
                self.is_completed = true;
                Some(format!("Tugas selesai: {answer}"))
            } else {
                let input_str = action_input.as_deref().unwrap_or("");
                let obs = tool_runner(act, input_str)?;
                Some(obs)
            }
        } else {
            None
        };

        let obs_snippet = observation.as_deref().unwrap_or("no_observation");
        circuit_breaker.record_step_outcome(&node_id, obs_snippet, estimated_tokens)?;

        let step_record = ReActStep {
            step: current_step_idx,
            thought,
            action,
            action_input,
            observation,
            temperature: temp,
            tokens_used: estimated_tokens,
        };

        self.steps.push(step_record.clone());

        if current_step_idx >= self.max_steps && !self.is_completed {
            self.is_completed = true;
            if self.final_answer.is_none() {
                self.final_answer = Some(format!("Batas langkah ({}) tercapai.", self.max_steps));
            }
        }

        Ok(step_record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_temperature_decay() {
        let session = ReActSession::new("Analisis Pasar".to_string(), 5, 0.7, 0.85);
        let t1 = session.calculate_decayed_temperature(1);
        let t2 = session.calculate_decayed_temperature(2);
        let t5 = session.calculate_decayed_temperature(5);

        assert!(t1 < 0.7);
        assert!(t2 < t1);
        assert!(t5 >= 0.10);
    }

    #[test]
    fn test_react_execution_loop() {
        let mut session = ReActSession::new("Kalkulasi Nilai".to_string(), 3, 0.6, 0.8);
        let mut cb = CircuitBreaker::new(5, 60, 5000);

        let step1 = session.execute_step(
            &mut cb,
            |act, input| Ok(format!("Hasil {act} pada {input}")),
            "Saya perlu mengambil data".to_string(),
            Some("fetch_data".to_string()),
            Some("key_1".to_string()),
            150,
        ).unwrap();

        assert_eq!(step1.step, 1);
        assert_eq!(step1.observation.unwrap(), "Hasil fetch_data pada key_1");

        let step2 = session.execute_step(
            &mut cb,
            |_act, _input| Ok("".to_string()),
            "Data telah diperoleh, kesimpulan siap".to_string(),
            Some("FinalAnswer".to_string()),
            Some("Nilai = 42".to_string()),
            100,
        ).unwrap();

        assert_eq!(step2.step, 2);
        assert!(session.is_completed);
        assert_eq!(session.final_answer.as_deref(), Some("Nilai = 42"));
    }
}
