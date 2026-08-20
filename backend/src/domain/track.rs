/// A track's time window must start at or after zero, and be non-empty
/// (`end_ms` strictly after `start_ms`). The "after zero" half is only
/// enforced here — the `tracks_end_after_start` CHECK constraint in
/// `migrations/0003_create_tracks.sql` covers the ordering half, but a
/// negative `start_ms` would otherwise slip through as long as `end_ms`
/// was also negative and still greater.
pub fn is_valid_time_range(start_ms: i32, end_ms: i32) -> bool {
    start_ms >= 0 && end_ms > start_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_positive_range() {
        assert!(is_valid_time_range(0, 100));
    }

    #[test]
    fn rejects_a_zero_length_range() {
        assert!(!is_valid_time_range(50, 50));
    }

    #[test]
    fn rejects_an_inverted_range() {
        assert!(!is_valid_time_range(100, 50));
    }

    #[test]
    fn rejects_a_negative_start() {
        assert!(!is_valid_time_range(-100, -50));
        assert!(!is_valid_time_range(-1, 100));
    }
}
