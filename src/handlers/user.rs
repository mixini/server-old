use axum::{body::Body, extract::Extension};
use http::{Response, StatusCode};
use lazy_static::lazy_static;
use lettre::Message;
use regex::Regex;
use serde::Deserialize;
use std::sync::Arc;
use ulid::Ulid;
use uuid::Uuid;
use validator::Validate;

use crate::error::MixiniError;
use crate::handlers::ValidatedForm;
use crate::server::State;
use crate::utils::pass::HASHER;

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9\.\-_]+$").unwrap();
    static ref RE_PASSWORD: Regex = Regex::new(r"^[a-zA-Z0-9]*[0-9][a-zA-Z0-9]*$").unwrap();
}

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
struct VerifyInput {
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

    let res = sqlx::query_as!(
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

// /// Handler for `POST /user/verify`
// pub(crate) async fn create_verify_req(
//     state: Extension<Arc<State>>,
// ) -> Result<Response<()>, MixiniError> {
//     unimplemented!()
// }

// pub(crate) async fn new_user(mut req: Request<State>) -> Result<Response> {
//     let form: RegisterForm = req.body_form().await?;

//     match form.validate() {
//         Ok(()) => {
//             // acquire state
//             let state = req.state();

//             state
//                 .redis_manager
//                 .clone()
//                 .set_ex(&full_key, encoded_reg_info, REGISTER_EXPIRY_SECONDS)
//                 .await?;

//             // send an email to the registrant with a code to verify
//             let email = Message::builder()
//                 .from(std::env::var("SMTP_EMAIL")?.parse()?)
//                 .to(reg_info.email.parse()?)
//                 .subject("Your Mixini registration verification")
//                 .body(format!(
//                     "Your Mixini registration key is {}. Note that it will expire in 24 hours.",
//                     key
//                 ))?;

//             state.mailer.send(&email)?;

//             // reply with OK
//             let res = Response::builder(StatusCode::Ok).build();
//             Ok(res)
//         }
//         Err(form_errors) => {
//             let res = Response::builder(StatusCode::BadRequest)
//                 .body(json!(form_errors))
//                 .build();
//             Ok(res)
//         }
//     }
// }

// /// Endpoint for `PUT /register/verify`
// pub(crate) async fn verify(req: Request<State>) -> Result<Response> {
//     let form: VerifyQuery = req.query()?;

//     match form.validate() {
//         Ok(()) => {
//             // acquire state
//             let state = req.state();
//             let key = form.key;
//             let full_key = format!("{}{}", REGISTER_KEY_PREFIX, &key);

//             let encoded_reg_info: Vec<u8> = state.redis_manager.clone().get(&full_key).await?;

//             // if it's empty, then the registration likely expired
//             if encoded_reg_info.is_empty() {
//                 let res = Response::builder(StatusCode::BadRequest).build();
//                 return Ok(res);
//             }

//             // insert a new user into the database with this info
//             let reg_info: RegistrationInfo = bincode::deserialize(&encoded_reg_info)?;

//             let mut db_conn = state.db_pool.acquire().await?;

//             sqlx::query_as!(
//                 User,
//                 r#"INSERT INTO users (name, email, password) VALUES ($1, $2, $3)"#,
//                 reg_info.name,
//                 reg_info.email,
//                 reg_info.password
//             )
//             .execute(&mut db_conn)
//             .await?;

//             // delete value at key
//             state.redis_manager.clone().del(&full_key).await?;

//             let res = Response::builder(StatusCode::Ok).build();
//             Ok(res)
//         }
//         Err(form_errors) => {
//             let res = Response::builder(StatusCode::BadRequest)
//                 .body(json!(form_errors))
//                 .build();
//             Ok(res)
//         }
//     }
// }
