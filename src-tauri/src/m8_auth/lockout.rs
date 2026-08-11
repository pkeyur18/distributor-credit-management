// D-2's lockout ladder: locks at 5 consecutive failures, then at every 5
// further failures, durations 30s -> 2min -> 10min -> 30min -> 1h (capped).
// The counter resets only on a successful login — never on lockout expiry
// (the prototype's flat-20s-with-reset was demo pacing, not a security
// decision; porting it would give a patient attacker unlimited batches of
// five). State lives in the sidecar file (`store::AuthStore`), so a process
// kill does not clear it.
use chrono::{DateTime, Utc};

/// `None` if this failure count doesn't cross a lockout threshold. `Some`
/// gives the duration of the newly-triggered lock.
pub fn tier_duration_seconds(failed_attempts: i64) -> Option<i64> {
    if failed_attempts < 5 || failed_attempts % 5 != 0 {
        return None;
    }
    let tier = (failed_attempts / 5).min(5);
    Some(match tier {
        1 => 30,
        2 => 120,
        3 => 600,
        4 => 1_800,
        _ => 3_600,
    })
}

pub fn locked_until_from_now(seconds: i64) -> String {
    (Utc::now() + chrono::Duration::seconds(seconds)).to_rfc3339()
}

/// Seconds remaining on an active lock, or `None` if unlocked (including a
/// stored timestamp that has already elapsed, or none stored at all).
pub fn seconds_remaining(locked_until: &Option<String>) -> Option<i64> {
    let locked_until: DateTime<Utc> = locked_until
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))?;
    let remaining = (locked_until - Utc::now()).num_seconds();
    if remaining > 0 {
        Some(remaining)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_lock_below_five_failures() {
        for n in 0..5 {
            assert_eq!(tier_duration_seconds(n), None, "attempt {n} must not lock");
        }
    }

    #[test]
    fn ladder_escalates_at_every_fifth_failure() {
        assert_eq!(tier_duration_seconds(5), Some(30));
        assert_eq!(tier_duration_seconds(10), Some(120));
        assert_eq!(tier_duration_seconds(15), Some(600));
        assert_eq!(tier_duration_seconds(20), Some(1_800));
        assert_eq!(tier_duration_seconds(25), Some(3_600));
    }

    #[test]
    fn ladder_caps_at_one_hour_beyond_tier_five() {
        assert_eq!(tier_duration_seconds(30), Some(3_600));
        assert_eq!(tier_duration_seconds(100), Some(3_600));
    }

    #[test]
    fn non_multiples_of_five_do_not_re_trigger_a_lock() {
        // Attempts 6-9 fail without a fresh lock; the next one is at 10.
        for n in [6, 7, 8, 9] {
            assert_eq!(tier_duration_seconds(n), None);
        }
    }

    #[test]
    fn seconds_remaining_is_none_once_elapsed() {
        let past = (Utc::now() - chrono::Duration::seconds(5)).to_rfc3339();
        assert_eq!(seconds_remaining(&Some(past)), None);
    }

    #[test]
    fn seconds_remaining_is_none_when_nothing_stored() {
        assert_eq!(seconds_remaining(&None), None);
    }

    #[test]
    fn seconds_remaining_counts_down_from_a_future_lock() {
        let future = locked_until_from_now(30);
        let remaining = seconds_remaining(&Some(future)).unwrap();
        assert!((1..=30).contains(&remaining), "got {remaining}");
    }
}
