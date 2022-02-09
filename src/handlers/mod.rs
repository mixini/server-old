use axum::{
    async_trait,
    extract::{Form, FromRequest, RequestParts},
    BoxError,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::MixiniError;

pub(crate) mod login;
pub(crate) mod user;

pub(crate) use login::*;
pub(crate) use user::*;

lazy_static! {
    pub(crate) static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9\.\-_]+$").unwrap();
    pub(crate) static ref RE_PASSWORD: Regex =
        Regex::new(r"^[a-zA-Z0-9]*[0-9][a-zA-Z0-9]*$").unwrap();
}

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
    type Rejection = MixiniError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}
