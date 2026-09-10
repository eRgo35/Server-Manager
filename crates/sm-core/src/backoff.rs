#[derive(Debug, Clone)]
pub struct BackoffState {
    base_secs: u64,
    cap_secs: u64,
    failures: u32,
}

impl BackoffState {
    pub fn new(base_secs: u64) -> Self {
        Self {
            base_secs: base_secs.max(1),
            cap_secs: 300,
            failures: 0,
        }
    }

    pub fn on_success(&mut self) {
        self.failures = 0;
    }

    pub fn on_failure(&mut self) {
        self.failures = self.failures.saturating_add(1);
    }

    pub fn current_delay_secs(&self) -> u64 {
        let factor = 2u64.saturating_pow(self.failures);
        self.base_secs.saturating_mul(factor).min(self.cap_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramps_and_caps_and_resets() {
        let mut b = BackoffState::new(5);
        assert_eq!(b.current_delay_secs(), 5);
        b.on_failure();
        assert_eq!(b.current_delay_secs(), 10);
        b.on_failure();
        assert_eq!(b.current_delay_secs(), 20);
        for _ in 0..20 {
            b.on_failure();
        }
        assert_eq!(b.current_delay_secs(), 300); // capped
        b.on_success();
        assert_eq!(b.current_delay_secs(), 5);
    }
}
