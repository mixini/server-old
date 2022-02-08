use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

// dost thou know of the pepeloni
const INTERNAL_SERVER_ERROR_MESSAGE: &str = "ahh the pepeloni";

/// Any possible errors
#[derive(Debug, Error)]
pub(crate) enum MixiniError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] axum::extract::rejection::FormRejection),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    RedisError(#[from] redis::RedisError),

    #[error(transparent)]
    LettreError(#[from] lettre::error::Error),

    #[error(transparent)]
    SmtpError(#[from] lettre::transport::smtp::Error),

    #[error(transparent)]
    BincodeError(#[from] bincode::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    OtherError(#[from] anyhow::Error),
}

impl IntoResponse for MixiniError {
    fn into_response(self) -> Response {
        match self {
            MixiniError::ValidationError(_) => {
                let message = format!("Input validation error: [{}]", self).replace("\n", ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            MixiniError::AxumFormRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            MixiniError::SqlxError(ref e) => match e {
                sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, self.to_string()),
                _ => {
                    tracing::debug!("Sqlx error occurred: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        INTERNAL_SERVER_ERROR_MESSAGE.into(),
                    )
                }
            },
            MixiniError::RedisError(e) => {
                tracing::debug!("Redis error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
            MixiniError::LettreError(e) => {
                tracing::debug!("Lettre error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
            MixiniError::SmtpError(e) => {
                tracing::debug!("Smtp error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
            MixiniError::BincodeError(e) => {
                tracing::debug!("Bincode error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
            MixiniError::JsonError(e) => {
                tracing::debug!("Json error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
            MixiniError::OtherError(e) => {
                tracing::debug!("Other error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
        }
        .into_response()
    }
}
