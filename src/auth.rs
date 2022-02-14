use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    headers::Cookie,
};
use redis::AsyncCommands;

use crate::{error::MixiniError, models::User, server::State};

pub(crate) const SESSION_COOKIE_NAME: &str = "msessid";
pub(crate) const SESSION_KEY_PREFIX: &str = "session:";

/// The authorization of a user making a request.
#[derive(Debug)]
pub(crate) enum Auth {
    KnownUser(User),
    UnknownUser,
}

#[async_trait]
impl<B> FromRequest<B> for Auth
where
    B: Send,
{
    type Rejection = MixiniError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(state) = Extension::<State>::from_request(req)
            .await
            .expect("State extension missing");

        let cookie = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .unwrap();

        match cookie
            .as_ref()
            .and_then(|cookie| cookie.get(SESSION_COOKIE_NAME))
        {
            Some(base_key) => {
                let prefixed_key = format!("{}{}", SESSION_KEY_PREFIX, &base_key);
                match state.redis_manager.clone().get(&prefixed_key).await? {
                    Some(user) => Ok(Auth::KnownUser(user)),
                    None => Ok(Auth::UnknownUser),
                }
            }
            None => Ok(Auth::UnknownUser),
        }
    }
}
