//! Auto-lock policy logic.
//!
//! Policies govern when the open vault should be cleared from memory.
//! The brief is explicit: do not wire full activity tracking here — the
//! check function lives on its own so future tasks can drive it from a
//! real "last input" timestamp without changing the public signature.

use std::time::{Duration, Instant};

/// Policy for when to lock the vault (i.e. clear it from memory).
///
/// `LockOnQuit` and `Disabled` are self-describing. `IdleTimeout`
/// holds the maximum allowed gap between the last user interaction and
/// the next policy evaluation before the vault is locked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoLockPolicy {
    /// Lock the vault on app quit. No idle timer.
    LockOnQuit,
    /// Lock the vault after the configured idle duration.
    IdleTimeout(Duration),
    /// Auto-lock is disabled entirely.
    Disabled,
}

/// Decide whether the vault should be locked right now.
///
/// * `LockOnQuit` -> never (callers handle quit separately).
/// * `Disabled` -> never.
/// * `IdleTimeout(d)` -> true iff `now - last_activity >= d`.
///
/// `last_activity` and `now` are passed in (rather than read from
/// `Instant::now()`) so unit tests can drive both edges of the timeout
/// without sleeping. A real activity tracker will be wired in a later
/// task; this signature is the contract that tracker has to honour.
pub fn should_lock_now(policy: &AutoLockPolicy, last_activity: Instant, now: Instant) -> bool {
    match policy {
        AutoLockPolicy::Disabled | AutoLockPolicy::LockOnQuit => false,
        AutoLockPolicy::IdleTimeout(duration) => {
            now.saturating_duration_since(last_activity) >= *duration
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{should_lock_now, AutoLockPolicy};
    use std::time::{Duration, Instant};

    #[test]
    fn disabled_never_locks() {
        let now = Instant::now();
        let last = now - Duration::from_secs(60);
        assert!(!should_lock_now(&AutoLockPolicy::Disabled, last, now));
    }

    #[test]
    fn lock_on_quit_never_locks_via_check() {
        // LockOnQuit is handled by the quit path, not by the idle check.
        let now = Instant::now();
        let last = now - Duration::from_secs(60);
        assert!(!should_lock_now(&AutoLockPolicy::LockOnQuit, last, now));
    }

    #[test]
    fn idle_timeout_locks_after_threshold() {
        let now = Instant::now();
        let last = now - Duration::from_secs(600);
        let policy = AutoLockPolicy::IdleTimeout(Duration::from_secs(300));
        assert!(should_lock_now(&policy, last, now));
    }

    #[test]
    fn idle_timeout_does_not_lock_before_threshold() {
        let now = Instant::now();
        let last = now - Duration::from_secs(60);
        let policy = AutoLockPolicy::IdleTimeout(Duration::from_secs(300));
        assert!(!should_lock_now(&policy, last, now));
    }

    #[test]
    fn idle_timeout_locks_exactly_at_threshold() {
        let now = Instant::now();
        let last = now - Duration::from_secs(300);
        let policy = AutoLockPolicy::IdleTimeout(Duration::from_secs(300));
        assert!(should_lock_now(&policy, last, now));
    }
}
