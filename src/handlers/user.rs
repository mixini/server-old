use anyhow::format_err;
use axum::{
    body::Body,
    extract::{Extension, Path, TypedHeader},
    headers::Cookie,
    http::{Response, StatusCode},
};
use entity::{prelude::*, sea_orm_active_enums::UserRole, user_account};
use fieldfilter::FieldFilterable;
use redis::AsyncCommands;
use sea_orm::{entity::*, prelude::*, query::*};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use ulid::Ulid;
use uuid::Uuid;
use validator::Validate;

use crate::{
    actions::{Delete, Read, UpdateUser},
    auth::Auth,
    constants::{
        RE_PASSWORD, RE_USERNAME, SESSION_COOKIE_NAME, SESSION_KEY_PREFIX, VERIFY_EXPIRY_SECONDS,
        VERIFY_KEY_PREFIX,
    },
    error::MixiniError,
    handlers::ValidatedForm,
    server::State,
    utils::{mail::send_email_verification_request, pass::HASHER, RKeys},
};

/// The form input for `POST /user`
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct CreateUser {
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

/// The form input for `PUT /user/verify`
#[derive(Debug, Validate, Deserialize)]
pub(crate) struct VerifyForm {
    #[validate(length(
        equal = 32,
        message = "Length of this key must be exactly 32 characters."
    ))]
    pub(crate) key: String,
}

/// The response for `GET /user/:id`
#[derive(Debug, Serialize, FieldFilterable)]
#[field_filterable_on(user_account::Model)]
pub(crate) struct GetUserResponse {
    id: Uuid,
    created_at: Option<DateTimeWithTimeZone>,
    updated_at: Option<DateTimeWithTimeZone>,
    name: Option<String>,
    email: Option<String>,
    role: Option<UserRole>,
}

/// Handler for `POST /user`
pub(crate) async fn create_user(
    ValidatedForm(create_user): ValidatedForm<CreateUser>,
    state: Extension<Arc<State>>,
) -> Result<Response<Body>, MixiniError> {
    // check if either this username or email already exist in our database
    let conflicts = UserAccount::find()
        .filter(
            Condition::any()
                .add(user_account::Column::Name.eq(create_user.name.to_owned()))
                .add(user_account::Column::Email.eq(create_user.email.to_owned())),
        )
        .all(&state.db)
        .await?;

    if !conflicts.is_empty() {
        Ok(Response::builder()
            .status(StatusCode::CONFLICT)
            .body(Body::from("A user with this name or email already exists."))
            .unwrap())
    } else {
        // create new user account in db
        let id = Uuid::from(Ulid::new());
        let password = HASHER
            .hash(&create_user.password)
            .expect("hasher failed hashing");

        let new_account = user_account::ActiveModel {
            id: Set(id),
            name: Set(create_user.name),
            email: Set(create_user.email),
            password: Set(password),
            ..Default::default()
        };
        new_account.insert(&state.db).await?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap())
    }
}

/// Handler for `GET /user/:id`
pub(crate) async fn get_user(
    Path(id): Path<Uuid>,
    state: Extension<Arc<State>>,
    auth: Auth,
) -> Result<Response<Body>, MixiniError> {
    let maybe_user = UserAccount::find_by_id(id).one(&state.db).await?;

    match maybe_user {
        Some(user) => {
            let authorized_fields: HashSet<String> = if let Auth::KnownUser(this_user) = auth {
                state
                    .oso
                    .lock()
                    .await
                    .authorized_fields(this_user, Read, user.to_owned())?
            } else {
                state
                    .oso
                    .lock()
                    .await
                    .authorized_fields("guest", Read, user.to_owned())?
            };

            let res_body = GetUserResponse::field_filter(user, authorized_fields);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(serde_json::to_vec(&res_body)?))
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
    ValidatedForm(update_user): ValidatedForm<UpdateUser>,
    state: Extension<Arc<State>>,
    auth: Auth,
) -> Result<Response<Body>, MixiniError> {
    match auth {
        Auth::KnownUser(this_user) => {
            let user = if let Some(user) = UserAccount::find_by_id(id).one(&state.db).await? {
                user
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::empty())
                    .unwrap());
            };

            if state.oso.lock().await.is_allowed(
                this_user,
                update_user.to_owned(),
                user.to_owned(),
            )? {
                // TODO: When UpdateUser -> ActiveModel works, change this
                // https://github.com/SeaQL/sea-orm/issues/547
                let mut user: user_account::ActiveModel = user.into();
                if let Some(name) = update_user.name {
                    user.name = Set(name);
                }
                if let Some(email) = update_user.email {
                    user.email = Set(email);
                }
                if let Some(role) = update_user.role {
                    user.role = Set(role);
                }
                user.update(&state.db).await?;
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::empty())
                    .unwrap())
            } else {
                Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
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

/// Handler for `DELETE /user/:id`
pub(crate) async fn delete_user(
    Path(id): Path<Uuid>,
    TypedHeader(cookie): TypedHeader<Cookie>,
    state: Extension<Arc<State>>,
    auth: Auth,
) -> Result<Response<Body>, MixiniError> {
    match auth {
        Auth::KnownUser(this_user) => {
            let user = if let Some(user) = UserAccount::find_by_id(id).one(&state.db).await? {
                user
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::empty())
                    .unwrap());
            };

            if state
                .oso
                .lock()
                .await
                .is_allowed(this_user, Delete, user.to_owned())?
            {
                user.delete(&state.db).await?;

                // also delete cookie in store
                let base_key = cookie.get(SESSION_COOKIE_NAME).expect("cookie monster!?");
                let prefixed_key = format!("{}{}", SESSION_KEY_PREFIX, &base_key);
                state.redis_manager.to_owned().del(&prefixed_key).await?;

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::empty())
                    .unwrap())
            } else {
                Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
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
                    .to_owned()
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
    ValidatedForm(verify): ValidatedForm<VerifyForm>,
    state: Extension<Arc<State>>,
) -> Result<Response<Body>, MixiniError> {
    // value is user id
    let prefixed_key = format!("{}{}", VERIFY_KEY_PREFIX, &verify.key);
    let maybe_id: Option<String> = state.redis_manager.to_owned().get(&prefixed_key).await?;

    match maybe_id {
        Some(id) => {
            let id: Uuid = Uuid::parse_str(&id).map_err(|e| format_err!(e))?;

            // NOTE: Normally this should always be Some(user) but better safe than sorry
            let user = if let Some(user) = UserAccount::find_by_id(id).one(&state.db).await? {
                user
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::empty())
                    .unwrap());
            };
            let mut user: user_account::ActiveModel = user.into();

            user.verified = Set(true);
            user.update(&state.db).await?;

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
