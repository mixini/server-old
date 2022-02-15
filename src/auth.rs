use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    headers::Cookie,
};
use chrono::Duration;
use lazy_static::lazy_static;
use redis::AsyncCommands;

use crate::{error::MixiniError, models::User, server::State};

pub(crate) const SESSION_COOKIE_NAME: &str = "msessid";
pub(crate) const SESSION_KEY_PREFIX: &str = "session:";
pub(crate) const SESSION_DURATION_SECS: usize = 1209600;

lazy_static! {
    pub(crate) static ref SESSION_DURATION: Duration =
        Duration::seconds(SESSION_DURATION_SECS as i64);
}

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

// An exit middleware fn that tells the client to refresh the expiry on the session cookie
// pub(crate) async fn map_refresh_session(
//     res: Response<Body>,
// ) -> Result<Response<Body>, MixiniError> {
//     //let headers = res.headers_mut();
//     todo!()
// }
