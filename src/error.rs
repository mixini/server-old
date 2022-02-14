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
    AxumFormRejection(#[from] axum::extract::rejection::FormRejection),

    #[error(transparent)]
    BincodeError(#[from] bincode::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    LettreError(#[from] lettre::error::Error),

    #[error(transparent)]
    OsoError(#[from] oso::OsoError),

    #[error(transparent)]
    RedisError(#[from] redis::RedisError),

    #[error(transparent)]
    SmtpError(#[from] lettre::transport::smtp::Error),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    OtherError(#[from] anyhow::Error),
}

impl IntoResponse for MixiniError {
    fn into_response(self) -> Response {
        match self {
            MixiniError::AxumFormRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
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
            MixiniError::LettreError(e) => {
                tracing::debug!("Lettre error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
            MixiniError::OsoError(e) => {
                tracing::debug!("Oso error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
            MixiniError::RedisError(e) => {
                tracing::debug!("Redis error occurred: {:?}", e);
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
            MixiniError::ValidationError(_) => {
                let message = format!("Input validation error: [{}]", self).replace("\n", ", ");
                (StatusCode::BAD_REQUEST, message)
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
