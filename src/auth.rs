use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
};
use redis::{
    AsyncCommands, ErrorKind as RedisErrorKind, FromRedisValue, RedisError, RedisResult,
    RedisWrite, ToRedisArgs, Value as RedisValue,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::MixiniError, server::State};

pub(crate) const SESSION_COOKIE_NAME: &str = "msessid";
pub(crate) const SESSION_KEY_PREFIX: &str = "session:";

#[derive(Debug)]
pub(crate) enum Auth {
    KnownUser(UserInfo),
    UnknownUser,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct UserInfo {
    pub(crate) id: Uuid,
    pub(crate) name: String,
}

impl ToRedisArgs for UserInfo {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(&bincode::serialize(self).expect("Failed to bincode-serialize user info"));
    }
}

impl FromRedisValue for UserInfo {
    fn from_redis_value(v: &RedisValue) -> RedisResult<Self> {
        if let RedisValue::Data(data) = v {
            Ok(bincode::deserialize(data).expect("Failed to bincode-deserialize user info"))
        } else {
            Err(RedisError::from((
                RedisErrorKind::TypeError,
                "Response type not convertible",
            )))
        }
    }
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

        let headers = req.headers().expect("another extractor took the headers");

        // TODO: use cookies instead

        // match headers
        //     .get(http::header::AUTHORIZATION)
        //     .and_then(|value| value.to_str().ok())
        //     .map(|value| value.to_string())
        // {
        //     Some(session_key) => {
        //         let qualified_key = format!("{}{}", SESSION_KEY_PREFIX, &session_key);
        //         let maybe_value: Option<Vec<u8>> =
        //             state.redis_manager.clone().get(&qualified_key).await?;

        //         if let Some(raw_user_info) = maybe_value {
        //             let user_info: UserInfo = bincode::deserialize(&raw_user_info)?;
        //             Ok(Auth::KnownUser(user_info))
        //         } else {
        //             Ok(Auth::UnknownUser)
        //         }
        //     }
        //     None => Ok(Auth::UnknownUser),
        // }
        todo!()
    }
}
