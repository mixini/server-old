use axum::{
    body::Body,
    extract::{Extension, TypedHeader},
    headers::Cookie,
    http::{header, Response, StatusCode},
};
use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use libreauth::pass::HashBuilder;
use redis::AsyncCommands;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use crate::auth::{SESSION_COOKIE_NAME, SESSION_KEY_PREFIX};
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
    static ref LOGIN_DURATION: Duration = Duration::days(30);
}

/// The form input of a `POST /user/login` request.
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct LoginForm {
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
    name: String,
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
    password: String,
}

/// Handler for `POST /login`
pub(crate) async fn login(
    ValidatedForm(input): ValidatedForm<LoginForm>,
    state: Extension<Arc<State>>,
) -> Result<Response<Body>, MixiniError> {
    let mut db_conn = state.db_pool.acquire().await?;

    let user = sqlx::query_as!(
        User,
        r#"SELECT id, created_at, updated_at, name, email, role as "role:_", password, verified
        FROM users WHERE name = $1"#,
        input.name
    )
    .fetch_one(&mut db_conn)
    .await?;

    let checker = HashBuilder::from_phc(&user.password).unwrap();

    if !checker.is_valid(&input.password) {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap());
    };
    if checker.needs_update(Some(PWD_SCHEME_VERSION)) {
        // password needs to be updated
        let hashed_password = HASHER
            .hash(&input.password)
            .expect("hasher somehow failed hashing");
        sqlx::query_as!(
            User,
            r#"UPDATE users SET password = $2 WHERE id = $1"#,
            user.id,
            hashed_password
        )
        .execute(&mut db_conn)
        .await?;
    }

    // create session entry in redis
    let RKeys {
        base_key,
        prefixed_key,
    } = RKeys::generate(SESSION_KEY_PREFIX);

    state.redis_manager.clone().set(&prefixed_key, user).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::SET_COOKIE,
            format!(
                "{cname}={cval}; Secure; HttpOnly; Domain={domain}; Expires={expiredate}",
                cname = SESSION_COOKIE_NAME,
                cval = base_key,
                domain = *DOMAIN,
                expiredate = (Utc::now() + *LOGIN_DURATION).to_rfc2822()
            ),
        )
        .body(Body::empty())
        .unwrap())
}

/// Handler for `DELETE /login`
pub(crate) async fn logout(
    TypedHeader(cookie): TypedHeader<Cookie>,
    state: Extension<Arc<State>>,
) -> Result<Response<Body>, MixiniError> {
    if let Some(sessid) = cookie.get(SESSION_COOKIE_NAME) {
        let prefixed_key = format!("{}{}", SESSION_KEY_PREFIX, sessid);
        state.redis_manager.clone().del(&prefixed_key).await?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(
                header::SET_COOKIE,
                format!(
                    "{cname}=expired; Secure; HttpOnly; Domain={domain}; Expires={expiredate}",
                    cname = SESSION_COOKIE_NAME,
                    domain = *DOMAIN,
                    expiredate = Utc::now().to_rfc2822()
                ),
            )
            .body(Body::empty())
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap())
    }
}
