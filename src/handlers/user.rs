use anyhow::format_err;
use axum::{body::Body, extract::Extension};
use http::{Response, StatusCode};
use lettre::{Message, Transport};
use redis::AsyncCommands;
use serde::Deserialize;
use std::sync::Arc;
use ulid::Ulid;
use uuid::Uuid;
use validator::Validate;

use crate::auth::Auth;
use crate::error::MixiniError;
use crate::handlers::{ValidatedForm, RE_PASSWORD, RE_USERNAME};
use crate::models::User;
use crate::server::State;
use crate::utils::{generate_redis_key, pass::HASHER};

const VERIFY_KEY_PREFIX: &str = "verify:";
const VERIFY_EXPIRY_SECONDS: usize = 86400;

/// The form input of a `POST /user` request.
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct NewUserInput {
    /// The provided username.
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
    /// The provided email.
    #[validate(email(message = "Must be a valid email address."))]
    pub(crate) email: String,
    /// The provided password.
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

/// The form input of a `PUT /user/verify` request.
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct VerifyInput {
    #[validate(length(
        equal = 32,
        message = "Length of this key must be exactly 32 characters."
    ))]
    key: String,
}

/// Handler for `POST /user`
pub(crate) async fn create_user(
    ValidatedForm(input): ValidatedForm<NewUserInput>,
    state: Extension<Arc<State>>,
) -> Result<Response<Body>, MixiniError> {
    // check if either this username or email already exist in our database
    let mut db_conn = state.db_pool.acquire().await?;

    // shadow
    let (name, email, password) = (input.name, input.email, input.password);

    let conflicts = sqlx::query!(
        r#"SELECT id FROM users WHERE name = $1 OR email = $2"#,
        name,
        email,
    )
    .fetch_optional(&mut db_conn)
    .await?;

    if conflicts.is_some() {
        let res = Response::builder()
            .status(StatusCode::CONFLICT)
            .body(Body::from("A user with this name or email already exists."))
            .unwrap();
        return Ok(res);
    }

    // create new user in db
    let id = Uuid::from(Ulid::new());
    let password = HASHER.hash(&password).unwrap();

    sqlx::query_as!(
        User,
        r#"INSERT INTO users (id, name, email, password) VALUES ($1, $2, $3, $4)"#,
        id,
        name,
        email,
        password,
    )
    .execute(&mut db_conn)
    .await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}

/// Handler for `POST /user/verify`
pub(crate) async fn create_verify_entry(
    state: Extension<Arc<State>>,
    auth: Auth,
) -> Result<Response<Body>, MixiniError> {
    match auth {
        Auth::KnownUser(user_info) => {
            let mut db_conn = state.db_pool.acquire().await?;

            let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_info.id)
                .fetch_one(&mut db_conn)
                .await?;

            if user.verified {
                Ok(Response::builder()
                    .status(StatusCode::CONFLICT)
                    .body(Body::from("User email is already verified"))
                    .unwrap())
            } else {
                let key = generate_redis_key(VERIFY_KEY_PREFIX);

                state
                    .redis_manager
                    .clone()
                    .set_ex(&key, user.id.to_string(), VERIFY_EXPIRY_SECONDS)
                    .await?;

                let mail = Message::builder()
                    .from(
                        std::env::var("SMTP_EMAIL")
                            .unwrap()
                            .parse()
                            .expect("SMTP_EMAIL key is invalid"),
                    )
                    .to(user.email.parse().unwrap())
                    .subject("Your Mixini email verification")
                    .body(format!(
                        "Your Mixini verification key is {}. Note that it will expire in 24 hours.",
                        key
                    ))?;

                state.mailer.send(&mail)?;

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::empty())
                    .unwrap())
            }
        }
        Auth::UnknownUser => Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap()),
    }
}

/// Handler for `PUT /user/verify`
pub(crate) async fn update_verify_user(
    ValidatedForm(input): ValidatedForm<VerifyInput>,
    state: Extension<Arc<State>>,
) -> Result<Response<Body>, MixiniError> {
    let maybe_id: Option<String> = state.redis_manager.clone().get(&input.key).await?;

    match maybe_id {
        Some(id) => {
            let id: Uuid = Uuid::parse_str(&id).map_err(|e| format_err!(e))?;

            let mut db_conn = state.db_pool.acquire().await?;

            sqlx::query_as!(User, "UPDATE users SET verified = TRUE WHERE id = $1", id)
                .execute(&mut db_conn)
                .await?;

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap())
        }
        None => Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap()),
    }
}
