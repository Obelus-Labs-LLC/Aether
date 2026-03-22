use std::time::Instant;

use crate::error::{AetherError, Result};
use crate::hcm::clock::MonotonicClock;
use crate::types::hcm::{HcmActivation, HcmState};

/// Manages HCM lifecycle: activation, renewal, scope enforcement, and exit.
pub struct HcmManager<C: MonotonicClock> {
    pub(crate) clock: C,
    state: HcmState,
}

impl<C: MonotonicClock> HcmManager<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            state: HcmState::default(),
        }
    }

    pub fn state(&self) -> &HcmState {
        &self.state
    }

    /// Activate HCM with the given activation event.
    /// If HCM is already active, this is treated as a renewal (per spec: no concurrent activations).
    pub fn activate(&mut self, activation: HcmActivation) -> Result<()> {
        if self.state.active {
            // Treat as renewal — extend the existing activation
            return self.renew(activation);
        }

        self.state.active = true;
        self.state.started_monotonic = Some(self.clock.now());
        self.state.activation = Some(activation);
        self.state.renewal_count = 0;
        Ok(())
    }

    /// Renew an active HCM session.
    fn renew(&mut self, activation: HcmActivation) -> Result<()> {
        if !self.state.active {
            return Err(AetherError::Hcm("cannot renew inactive HCM".to_string()));
        }

        let current = self.state.activation.as_ref().unwrap();
        if !current.allow_renewal {
            return Err(AetherError::Hcm("renewal not permitted".to_string()));
        }

        // Accumulate elapsed time
        if let Some(start) = self.state.started_monotonic {
            let elapsed = self.clock.elapsed_since(start);
            self.state.cumulative_seconds += elapsed.as_secs();
        }

        // Check cumulative limit
        let max_total = activation.max_total_duration_seconds;
        if self.state.cumulative_seconds >= max_total {
            return Err(AetherError::Hcm(format!(
                "cumulative HCM duration {}s exceeds maximum {}s",
                self.state.cumulative_seconds, max_total
            )));
        }

        self.state.started_monotonic = Some(self.clock.now());
        self.state.activation = Some(activation);
        self.state.renewal_count += 1;
        Ok(())
    }

    /// Deactivate HCM manually.
    pub fn deactivate(&mut self) -> Result<()> {
        if !self.state.active {
            return Err(AetherError::Hcm("HCM is not active".to_string()));
        }

        if let Some(start) = self.state.started_monotonic {
            let elapsed = self.clock.elapsed_since(start);
            self.state.cumulative_seconds += elapsed.as_secs();
        }

        self.state.active = false;
        self.state.started_monotonic = None;
        Ok(())
    }

    /// Check if HCM has expired (time limit exceeded).
    /// Returns true if HCM was active and has been auto-deactivated.
    pub fn check_expiry(&mut self) -> bool {
        if !self.state.active {
            return false;
        }

        let activation = match &self.state.activation {
            Some(a) => a,
            None => return false,
        };
        let start = match self.state.started_monotonic {
            Some(s) => s,
            None => return false,
        };

        let elapsed = self.clock.elapsed_since(start);

        // Check per-activation limit
        if elapsed.as_secs() >= activation.max_duration_seconds {
            let _ = self.deactivate();
            return true;
        }

        // Check cumulative limit
        let total = self.state.cumulative_seconds + elapsed.as_secs();
        if total >= activation.max_total_duration_seconds {
            let _ = self.deactivate();
            return true;
        }

        false
    }

    /// Check if a label_id is within the current HCM scope.
    pub fn is_in_scope(&self, label_id: &str) -> bool {
        self.state.is_in_scope(label_id)
    }

    /// Get the monotonic start instant (for elapsed calculation).
    pub fn started_at(&self) -> Option<Instant> {
        self.state.started_monotonic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hcm::clock::MockClock;
    use crate::types::hcm::ActorType;
    use std::time::Duration;

    fn make_activation() -> HcmActivation {
        HcmActivation {
            event_type: "hcm_activation".to_string(),
            event_id: "evt-001".to_string(),
            actor_id: "operator-1".to_string(),
            actor_type: ActorType::HumanOperator,
            timestamp: chrono::Utc::now(),
            reason: "Test activation".to_string(),
            scope: vec!["emergency".to_string(), "medical".to_string()],
            authorization_method: "manual".to_string(),
            max_duration_seconds: 14400,       // 4 hours
            max_total_duration_seconds: 259200, // 72 hours
            allow_renewal: true,
        }
    }

    #[test]
    fn basic_activation() {
        let clock = MockClock::new();
        let mut mgr = HcmManager::new(clock);
        mgr.activate(make_activation()).unwrap();
        assert!(mgr.state().active);
    }

    #[test]
    fn scope_enforcement() {
        let clock = MockClock::new();
        let mut mgr = HcmManager::new(clock);
        mgr.activate(make_activation()).unwrap();
        assert!(mgr.is_in_scope("emergency"));
        assert!(mgr.is_in_scope("medical"));
        assert!(!mgr.is_in_scope("routine"));
    }

    #[test]
    fn manual_deactivation() {
        let clock = MockClock::new();
        let mut mgr = HcmManager::new(clock);
        mgr.activate(make_activation()).unwrap();
        mgr.deactivate().unwrap();
        assert!(!mgr.state().active);
    }

    #[test]
    fn time_expiry() {
        let clock = MockClock::new();
        let mut mgr = HcmManager::new(clock);
        mgr.activate(make_activation()).unwrap();

        // Advance past 4-hour limit
        mgr.clock.advance(Duration::from_secs(14401));
        assert!(mgr.check_expiry());
        assert!(!mgr.state().active);
    }

    #[test]
    fn renewal() {
        let clock = MockClock::new();
        let mut mgr = HcmManager::new(clock);
        mgr.activate(make_activation()).unwrap();

        mgr.clock.advance(Duration::from_secs(3600)); // 1 hour

        // Activate again = renewal
        mgr.activate(make_activation()).unwrap();
        assert!(mgr.state().active);
        assert_eq!(mgr.state().renewal_count, 1);
        assert_eq!(mgr.state().cumulative_seconds, 3600);
    }

    #[test]
    fn cumulative_limit() {
        let clock = MockClock::new();
        let mut mgr = HcmManager::new(clock);

        let mut activation = make_activation();
        activation.max_total_duration_seconds = 100; // Very short cumulative limit

        mgr.activate(activation.clone()).unwrap();
        mgr.clock.advance(Duration::from_secs(50));
        mgr.deactivate().unwrap();

        // Re-activate
        mgr.activate(activation.clone()).unwrap();
        mgr.clock.advance(Duration::from_secs(51));

        // Should expire due to cumulative limit (50 + 51 > 100)
        assert!(mgr.check_expiry());
    }

    #[test]
    fn no_renewal_when_forbidden() {
        let clock = MockClock::new();
        let mut mgr = HcmManager::new(clock);

        let mut activation = make_activation();
        activation.allow_renewal = false;
        mgr.activate(activation.clone()).unwrap();

        // Try to activate again (renewal)
        activation.allow_renewal = false;
        assert!(mgr.activate(activation).is_err());
    }
}
