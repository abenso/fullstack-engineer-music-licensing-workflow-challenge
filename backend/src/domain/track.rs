/// A track's time window must be non-empty: `end_ms` strictly after `start_ms`.
/// Mirrors the `tracks_end_after_start` CHECK constraint in
/// `migrations/0003_create_tracks.sql` — kept here too so the API can
/// reject bad input with a clear 400 instead of a raw DB constraint error.
pub fn is_valid_time_range(start_ms: i32, end_ms: i32) -> bool {
    end_ms > start_ms
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
}
