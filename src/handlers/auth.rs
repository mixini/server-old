use axum::{body::Body, extract::Extension};
use http::{Response, StatusCode};
use libreauth::pass::HashBuilder;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::auth::{UserInfo, SESSION_KEY_PREFIX};
use crate::error::MixiniError;
use crate::handlers::{ValidatedForm, RE_PASSWORD, RE_USERNAME};
use crate::models::User;
use crate::server::State;
use crate::utils::{
    generate_redis_key,
    pass::{HASHER, PWD_SCHEME_VERSION},
};

// TODO: possibly rework this as `POST /user/login` and `DELETE /user/logout`, also rename auth to session

/// The form input of a `POST /auth` request.
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct NewAuthInput {
    #[validate(
        length(
            min = 5,
            max = 32,
            message = "Minimum length is 5 characters, maximum is 32"
        ),
        regex(
            path = "RE_USERNAME",
            message = "Can only contain letters, numbers, dashes (-), periods (.), and underscores (_)"
        )
    )]
    pub(crate) name: String,
    #[validate(
        length(
            min = 8,
            max = 128,
            message = "Minimum length is 8 characters, maximum is 128"
        ),
        regex(
            path = "RE_PASSWORD",
            message = "Must be alphanumeric and contain at least one number."
        )
    )]
    pub(crate) password: String,
}

/// The response of a `POST /auth` request.
///
/// Returns an "auth_key" meant to be used as an authorization header value in subsequent requests
#[derive(Debug, Serialize)]
pub(crate) struct NewAuthResponse {
    pub(crate) auth_key: String,
}

/// Handler for `POST /auth`
pub(crate) async fn create_auth(
    ValidatedForm(input): ValidatedForm<NewAuthInput>,
    state: Extension<Arc<State>>,
) -> Result<Response<Body>, MixiniError> {
    let mut db_conn = state.db_pool.acquire().await?;

    let (name, password) = (input.name, HASHER.hash(&input.password).unwrap());

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE name = $1", name)
        .fetch_one(&mut db_conn)
        .await?;

    let checker = HashBuilder::from_phc(&user.password).unwrap();

    if !checker.is_valid(&password) {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap());
    };
    if checker.needs_update(Some(PWD_SCHEME_VERSION)) {
        // password needs to be updated
        sqlx::query_as!(
            User,
            "UPDATE users SET password = $2 WHERE id = $1",
            user.id,
            password
        )
        .execute(&mut db_conn)
        .await?;
    }

    // create auth entry in redis
    let key = generate_redis_key(SESSION_KEY_PREFIX);
    let value: Vec<u8> = bincode::serialize(&UserInfo {
        id: user.id,
        name: user.name,
    })?;

    state.redis_manager.clone().set(&key, value).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(serde_json::ser::to_vec(&NewAuthResponse {
            auth_key: key,
        })?))
        .unwrap())
}
