use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::domain::license::LicenseStatus;

/// Single place that turns domain/DB failures into HTTP responses, so
/// handlers can just use `?` instead of matching errors themselves.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,

    #[error("{0}")]
    BadRequest(String),

    #[error("cannot move license status from {from:?} to {to:?}")]
    InvalidTransition {
        from: LicenseStatus,
        to: LicenseStatus,
    },

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidTransition { .. } => (StatusCode::CONFLICT, self.to_string()),
            AppError::Database(err) => {
                tracing::error!(error = %err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_404() {
        assert_eq!(
            AppError::NotFound.into_response().status(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn bad_request_maps_to_400() {
        let error = AppError::BadRequest("bad input".to_string());
        assert_eq!(error.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn invalid_transition_maps_to_409() {
        let error = AppError::InvalidTransition {
            from: LicenseStatus::Draft,
            to: LicenseStatus::Licensed,
        };
        assert_eq!(error.into_response().status(), StatusCode::CONFLICT);
    }

    #[test]
    fn database_error_maps_to_500() {
        let error = AppError::from(sqlx::Error::RowNotFound);
        assert_eq!(
            error.into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
