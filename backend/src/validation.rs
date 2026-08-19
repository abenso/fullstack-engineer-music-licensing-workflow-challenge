use crate::error::AppError;

/// Rejects a required text field that's empty or only whitespace, with a
/// `400` naming the field — used for the handful of "required text" inputs
/// (movie title, scene name, song title/artist/rights_holder) instead of
/// silently accepting blank values.
pub fn require_non_blank(field: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::BadRequest(format!("{field} must not be blank")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_non_blank_text() {
        assert!(require_non_blank("title", "Rocky IV").is_ok());
    }

    #[test]
    fn rejects_an_empty_string() {
        assert!(require_non_blank("title", "").is_err());
    }

    #[test]
    fn rejects_whitespace_only() {
        assert!(require_non_blank("title", "   ").is_err());
    }
}
