use async_trait::async_trait;
use axum::{
    extract::{Form, FromRequest, RequestParts},
    BoxError,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::ServerError;

pub(crate) mod auth;
pub(crate) mod user;

pub(crate) use user::new_user;

/// A validated form with some input.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ValidatedForm<T>(pub(crate) T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = ServerError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}
