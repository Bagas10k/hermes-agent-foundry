use std::time::Instant;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CircuitBreaker {
    pub max_steps: usize,
    pub timeout_seconds: u64,
    pub token_budget: u64,
    pub current_step: usize,
    pub tokens_used: u64,
    pub start_time: Instant,
    pub action_history: Vec<String>,
    pub is_tripped: bool,
    pub trip_reason: Option<String>,
}

#[allow(dead_code)]
impl CircuitBreaker {
    pub fn new(max_steps: usize, timeout_seconds: u64, token_budget: u64) -> Self {
        Self {
            max_steps: if max_steps == 0 { 10 } else { max_steps },
            timeout_seconds: if timeout_seconds == 0 { 180 } else { timeout_seconds },
            token_budget: if token_budget == 0 { 15000 } else { token_budget },
            current_step: 0,
            tokens_used: 0,
            start_time: Instant::now(),
            action_history: Vec::new(),
            is_tripped: false,
            trip_reason: None,
        }
    }

    pub fn check_before_step(&mut self, _node_id: &str) -> Result<(), String> {
        if self.is_tripped {
            return Err(format!("Circuit Breaker aktif [{}]. Eksekusi dibatalkan.", self.trip_reason.as_deref().unwrap_or("UNKNOWN")));
        }

        if self.start_time.elapsed().as_secs() > self.timeout_seconds {
            self.trip("TIMEOUT_EXCEEDED", format!("Batas waktu eksekusi {}s terlampaui", self.timeout_seconds));
            return Err(self.trip_reason.clone().unwrap());
        }

        self.current_step += 1;
        if self.current_step > self.max_steps {
            self.trip("MAX_STEPS_EXCEEDED", format!("Jumlah langkah eksekusi melebihi batas maksimum ({} langkah)", self.max_steps));
            return Err(self.trip_reason.clone().unwrap());
        }

        if self.tokens_used >= self.token_budget {
            self.trip("TOKEN_BUDGET_EXCEEDED", format!("Plafon konsumsi token ({}) telah habis", self.token_budget));
            return Err(self.trip_reason.clone().unwrap());
        }

        Ok(())
    }

    pub fn record_step_outcome(&mut self, node_id: &str, output_snippet: &str, tokens_consumed: u64) -> Result<(), String> {
        self.tokens_used += tokens_consumed;

        let fingerprint = format!("{}:{}", node_id, &output_snippet[..output_snippet.len().min(80)]);
        self.action_history.push(fingerprint);

        let len = self.action_history.len();
        if len >= 3 && self.action_history[len - 1] == self.action_history[len - 2] && self.action_history[len - 2] == self.action_history[len - 3] {
            self.trip("THRASHING_LOOP_DETECTED", format!("Mendeteksi perulangan aksi identik pada modul '{node_id}' sebanyak 3 kali"));
            return Err(self.trip_reason.clone().unwrap());
        }

        if self.tokens_used >= self.token_budget {
            self.trip("TOKEN_BUDGET_EXCEEDED", format!("Plafon konsumsi token ({}) telah habis", self.token_budget));
            return Err(self.trip_reason.clone().unwrap());
        }

        Ok(())
    }

    fn trip(&mut self, reason: &str, message: String) {
        self.is_tripped = true;
        self.trip_reason = Some(format!("{reason}: {message}"));
    }
}
