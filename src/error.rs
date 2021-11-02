use axum::{
    body::{Bytes, Full},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use std::convert::Infallible;
use thiserror::Error;

// dost thou know of the pepeloni
const INTERNAL_SERVER_ERROR_MESSAGE: &str = "aah the pepeloni";

/// Any possible server errors
#[derive(Debug, Error)]
pub(crate) enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] axum::extract::rejection::FormRejection),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
}

impl IntoResponse for ServerError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        match self {
            ServerError::ValidationError(_) => {
                let message = format!("Input validation error: [{}]", self).replace("\n", ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            ServerError::AxumFormRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::SqlxError(e) => {
                tracing::debug!("Sqlx error occurred: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    INTERNAL_SERVER_ERROR_MESSAGE.into(),
                )
            }
        }
        .into_response()
    }
}
