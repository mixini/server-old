use axum::{
    async_trait,
    body::HttpBody,
    extract::{Form, FromRequest, RequestParts},
    BoxError,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::MixiniError;

pub mod login;
pub mod user;

pub use login::*;
pub use user::*;

/// A validated form with some input.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = MixiniError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}
