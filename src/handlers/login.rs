use axum::{body::Body, extract::Extension};
use http::{header, Response, StatusCode};
use lazy_static::lazy_static;
use libreauth::pass::HashBuilder;
use redis::AsyncCommands;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use crate::auth::{UserInfo, SESSION_COOKIE_NAME, SESSION_KEY_PREFIX};
use crate::error::MixiniError;
use crate::handlers::{ValidatedForm, RE_PASSWORD, RE_USERNAME};
use crate::models::User;
use crate::server::State;
use crate::utils::{
    pass::{HASHER, PWD_SCHEME_VERSION},
    RKeys,
};

lazy_static! {
    static ref DOMAIN: String = std::env::var("DOMAIN").expect("DOMAIN is not set in env");
}

// TODO: possibly rework this as `POST /user/login` and `DELETE /user/logout`, also rename auth to session

/// The form input of a `POST /user/login` request.
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct LoginInput {
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

// /// The response of a `POST /auth` request.
// ///
// /// Returns an "auth_key" meant to be used as an authorization header value in subsequent requests
// #[derive(Debug, Serialize)]
// pub(crate) struct LoginResponse {
//     pub(crate) auth_key: String,
// }

/// Handler for `POST /auth`
pub(crate) async fn create_auth(
    ValidatedForm(input): ValidatedForm<LoginInput>,
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

    // create session entry in redis
    let RKeys {
        base_key,
        prefixed_key,
    } = RKeys::generate(SESSION_KEY_PREFIX);

    let user_info = UserInfo {
        id: user.id,
        name: user.name,
    };

    state
        .redis_manager
        .clone()
        .set(&prefixed_key, user_info)
        .await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::SET_COOKIE,
            format!(
                "{0}={1}; Secure; HttpOnly; Domain={2}",
                SESSION_COOKIE_NAME, base_key, *DOMAIN
            ),
        )
        .body(Body::empty())
        .unwrap())
}
