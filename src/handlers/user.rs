use anyhow::format_err;
use axum::{
    body::Body,
    extract::{Extension, Path},
    http::{Response, StatusCode},
};
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use ulid::Ulid;
use uuid::Uuid;
use validator::Validate;

use crate::error::MixiniError;
use crate::handlers::{ValidatedForm, RE_PASSWORD, RE_USERNAME};
use crate::models::User;
use crate::server::State;
use crate::utils::{mail::send_email_verification_request, pass::HASHER, RKeys};
use crate::{auth::Auth, models::Role};

const VERIFY_KEY_PREFIX: &str = "verify:";
const VERIFY_EXPIRY_SECONDS: usize = 86400;

/// The form input for `POST /user`
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct CreateUserForm {
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
    name: String,
    /// The provided email.
    #[validate(email(message = "Must be a valid email address."))]
    email: String,
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
    password: String,
}

/// The form input for `PUT /user/:id`
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct UpdateUserForm {
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
    name: Option<String>,
    #[validate(email(message = "Must be a valid email address."))]
    email: Option<String>,
    role: Option<Role>,
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
    password: Option<String>,
}

/// The form input for `PUT /user/verify`
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct VerifyForm {
    #[validate(length(
        equal = 32,
        message = "Length of this key must be exactly 32 characters."
    ))]
    key: String,
}

/// The response output for `GET /user/:name`
#[derive(Debug, Serialize)]
pub(crate) struct UserResponse {
    id: Uuid,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    name: String,
    email: Option<String>,
    role: Role,
}

/// Handler for `POST /user`
pub(crate) async fn create_user(
    ValidatedForm(input): ValidatedForm<CreateUserForm>,
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

/// Handler for `GET /user/:id`
pub(crate) async fn get_user(
    Path(id): Path<Uuid>,
    state: Extension<Arc<State>>,
    auth: Auth,
) -> Result<Response<Body>, MixiniError> {
    let mut db_conn = state.db_pool.acquire().await?;

    match sqlx::query_as!(
        User,
        r#"SELECT id, created_at, updated_at, name, email, role as "role:_", password, verified
        FROM users WHERE id = $1"#,
        id
    )
    .fetch_optional(&mut db_conn)
    .await?
    {
        Some(user) => {
            let authorized_fields: HashSet<String> = if let Auth::KnownUser(this_user) = auth {
                state
                    .oso
                    .lock()
                    .await
                    .authorized_fields(this_user, "READ", user.clone())?
            } else {
                state
                    .oso
                    .lock()
                    .await
                    .authorized_fields("guest", "READ", user.clone())?
            };

            let created_at = if authorized_fields.contains("created_at") {
                Some(user.created_at)
            } else {
                None
            };
            let updated_at = if authorized_fields.contains("updated_at") {
                Some(user.updated_at)
            } else {
                None
            };
            let email = if authorized_fields.contains("email") {
                Some(user.email)
            } else {
                None
            };

            let user_response = UserResponse {
                id: user.id,
                created_at,
                updated_at,
                name: user.name,
                email,
                role: user.role,
            };
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(serde_json::to_vec(&user_response)?))
                .unwrap())
        }
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}

/// Handler for `PUT /user/:id`
pub(crate) async fn update_user(
    Path(id): Path<Uuid>,
    ValidatedForm(input): ValidatedForm<UpdateUserForm>,
    state: Extension<Arc<State>>,
    auth: Auth,
) -> Result<Response<Body>, MixiniError> {
    match auth {
        Auth::KnownUser(this_user) => {
            let mut db_conn = state.db_pool.acquire().await?;

            let user = if let Some(user) = sqlx::query_as!(
                    User,
                    r#"SELECT id, created_at, updated_at, name, email, role as "role:_", password, verified
                    FROM users WHERE id = $1"#,
                    id
                )
                .fetch_optional(&mut db_conn)
                .await? { user } else {
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::empty())
                        .unwrap());
                };

            let mut values = Vec::new();

            if let Some(role) = input.role {
                if state
                    .oso
                    .lock()
                    .await
                    .query_rule("allow_assign_role", (this_user.clone(), role))?
                    .next()
                    .is_some()
                {
                    values.push(format!("role = '{}'", role));
                } else {
                    return Ok(Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::empty())
                        .unwrap());
                }
            }

            let authorized_fields: HashSet<String> =
                state
                    .oso
                    .lock()
                    .await
                    .authorized_fields(this_user, "READ", user.clone())?;

            if let Some(name) = input.name {
                if authorized_fields.contains("name") {
                    values.push(format!("name = {}", name));
                }
            }

            if let Some(email) = input.email {
                if authorized_fields.contains("email") {
                    values.push(format!("email = {}", email));
                }
            }

            let values = values.join(",");

            // TODO: Required testing, this is a dynamic query
            sqlx::query(r#"UPDATE users SET $2 WHERE id = $1"#)
                .bind(id)
                .bind(values)
                .execute(&mut db_conn)
                .await?;

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap())
        }
        Auth::UnknownUser => Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap()),
    }
}

/// Handler for `POST /user/verify`
pub(crate) async fn create_verify_user(
    state: Extension<Arc<State>>,
    auth: Auth,
) -> Result<Response<Body>, MixiniError> {
    match auth {
        Auth::KnownUser(this_user) => {
            if this_user.verified {
                Ok(Response::builder()
                    .status(StatusCode::CONFLICT)
                    .body(Body::from("User email is already verified"))
                    .unwrap())
            } else {
                let RKeys {
                    base_key,
                    prefixed_key,
                } = RKeys::generate(VERIFY_KEY_PREFIX);

                state
                    .redis_manager
                    .clone()
                    .set_ex(
                        &prefixed_key,
                        this_user.id.to_string(),
                        VERIFY_EXPIRY_SECONDS,
                    )
                    .await?;

                send_email_verification_request(&state.mailsender, this_user.email, base_key)
                    .await?;

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
    ValidatedForm(input): ValidatedForm<VerifyForm>,
    state: Extension<Arc<State>>,
) -> Result<Response<Body>, MixiniError> {
    // value is user id
    let prefixed_key = format!("{}{}", VERIFY_KEY_PREFIX, &input.key);
    let maybe_id: Option<String> = state.redis_manager.clone().get(&prefixed_key).await?;

    match maybe_id {
        Some(id) => {
            let id: Uuid = Uuid::parse_str(&id).map_err(|e| format_err!(e))?;

            let mut db_conn = state.db_pool.acquire().await?;

            sqlx::query_as!(
                User,
                r#"UPDATE users SET verified = TRUE WHERE id = $1"#,
                id
            )
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
