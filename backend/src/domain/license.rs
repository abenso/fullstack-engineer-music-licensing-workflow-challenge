use serde::{Deserialize, Serialize};

/// Status of the licensing negotiation for a single track.
///
/// Mirrors the `license_status` enum in `migrations/0003_create_tracks.sql` —
/// keep both in sync. `rename_all` controls two independent encodings:
/// snake_case for the Postgres enum, PascalCase for the JSON API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "license_status", rename_all = "snake_case")]
#[serde(rename_all = "PascalCase")]
pub enum LicenseStatus {
    Draft,
    Requested,
    InNegotiation,
    Approved,
    Licensed,
    Rejected,
}

/// Returns `true` if moving a track's license from `from` to `to` is a legal
/// step in the negotiation workflow.
///
/// `Licensed` and `Rejected` are terminal: no transition starts from them.
pub fn can_transition(from: LicenseStatus, to: LicenseStatus) -> bool {
    use LicenseStatus::*;

    matches!(
        (from, to),
        (Draft, Requested)
            | (Requested, InNegotiation)
            | (Requested, Rejected)
            | (InNegotiation, Approved)
            | (InNegotiation, Rejected)
            | (Approved, Licensed)
    )
}

#[cfg(test)]
mod tests {
    use super::LicenseStatus::*;
    use super::*;

    #[test]
    fn allows_the_happy_path_step_by_step() {
        assert!(can_transition(Draft, Requested));
        assert!(can_transition(Requested, InNegotiation));
        assert!(can_transition(InNegotiation, Approved));
        assert!(can_transition(Approved, Licensed));
    }

    #[test]
    fn allows_rejection_from_requested_or_in_negotiation() {
        assert!(can_transition(Requested, Rejected));
        assert!(can_transition(InNegotiation, Rejected));
    }

    #[test]
    fn rejects_skipping_steps() {
        assert!(!can_transition(Draft, Approved));
        assert!(!can_transition(Draft, Licensed));
        assert!(!can_transition(Requested, Licensed));
    }

    #[test]
    fn rejects_moving_out_of_terminal_states() {
        assert!(!can_transition(Licensed, Draft));
        assert!(!can_transition(Rejected, Requested));
    }

    #[test]
    fn rejects_moving_backwards() {
        assert!(!can_transition(InNegotiation, Requested));
        assert!(!can_transition(Approved, InNegotiation));
    }

    #[test]
    fn rejects_staying_in_place() {
        assert!(!can_transition(Draft, Draft));
        assert!(!can_transition(Approved, Approved));
    }
}
