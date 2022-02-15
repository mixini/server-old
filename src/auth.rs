use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    headers::Cookie,
};
use redis::AsyncCommands;

use crate::{
    constants::{SESSION_COOKIE_NAME, SESSION_DURATION_SECS, SESSION_KEY_PREFIX},
    error::MixiniError,
    models::User,
    server::State,
};

/// The authorization of a user making a request.
///
/// The extractor middleware that captures this Auth looks for a `SESSION_COOKIE_NAME` cookie
/// with the value being the unprefixed key.
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
                let mut redis_manager = state.redis_manager.to_owned();
                match redis_manager.get(&prefixed_key).await? {
                    Some(user) => {
                        // refresh expiry on key
                        redis_manager
                            .expire(&prefixed_key, SESSION_DURATION_SECS)
                            .await?;
                        Ok(Auth::KnownUser(user))
                    }
                    None => Ok(Auth::UnknownUser),
                }
            }
            None => Ok(Auth::UnknownUser),
        }
    }
}
