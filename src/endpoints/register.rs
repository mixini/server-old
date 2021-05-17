use lazy_static::lazy_static;
use lettre::{Message, Transport};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use redis::AsyncCommands;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tide::{convert::json, Request, Response, Result, StatusCode};
use validator::Validate;

use crate::server::State;
use crate::utils::pass::HASHER;

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9\.\-_]+$").unwrap();
    static ref RE_PASSWORD: Regex = Regex::new(r"^[a-zA-Z0-9]*[0-9][a-zA-Z0-9]*$").unwrap();
}

const REGISTER_KEY_PREFIX: &str = "register:";
const REGISTER_EXPIRY_SECONDS: usize = 86400;

/// The form of a `POST /register` request.
#[derive(Debug, Validate, Deserialize)]
struct RegisterForm {
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

/// A local, intermediate structure for storing registration info, distinct from `RegisterForm` in that the password is hashed, so the plaintext password is never stored.
#[derive(Debug, Serialize, Deserialize)]
struct RegistrationInfo {
    /// The username.
    name: String,
    /// The email.
    email: String,
    /// The password in hashed PHC form.
    password: String,
}

#[derive(Debug, Validate, Deserialize)]
/// The form of a `POST /verify` request.
struct VerifyForm {
    #[validate(length(
        equal = 32,
        message = "Length of this key must be exactly 32 characters."
    ))]
    key: String,
}

/// Endpoint for `POST /register`
pub(crate) async fn register(mut req: Request<State>) -> Result<Response> {
    let form: RegisterForm = req.body_form().await?;

    match form.validate() {
        Ok(()) => {
            // acquire state
            let state = req.state();

            // check if either this username or email already exist in our database
            let mut db_conn = state.db_pool.acquire().await?;

            let conflicts = sqlx::query!(
                r#"SELECT id FROM users WHERE name = $1 OR email = $2"#,
                form.name,
                form.email,
            )
            .fetch_all(&mut db_conn)
            .await?;

            if !conflicts.is_empty() {
                let res = Response::builder(StatusCode::UnprocessableEntity)
                    .body(json!({"error": "A user with this name or email already exists."}))
                    .build();
                return Ok(res);
            }

            // record this registration in redis
            let key: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();
            let full_key = format!("{}{}", REGISTER_KEY_PREFIX, &key);

            // hash the password
            let hashed_pw = match HASHER.hash(&form.password) {
                Ok(s) => s,
                Err(e) => {
                    tide::log::error!("Error occurred with hasher: {:?}", e);
                    let res = Response::builder(StatusCode::InternalServerError).build();
                    return Ok(res);
                }
            };

            // encode this registration info
            let reg_info = RegistrationInfo {
                name: form.name,
                email: form.email,
                password: hashed_pw,
            };
            let encoded_reg_info: Vec<u8> = bincode::serialize(&reg_info)?;

            state
                .redis_manager
                .clone()
                .set_ex(&full_key, encoded_reg_info, REGISTER_EXPIRY_SECONDS)
                .await?;

            // send an email to the registrant with a code to verify
            let email = Message::builder()
                .from(std::env::var("SMTP_EMAIL")?.parse()?)
                .to(reg_info.email.parse()?)
                .subject("Your Mixini registration verification")
                .body(format!(
                    "Your Mixini registration key is {}. Note that it will expire in 24 hours.",
                    key
                ))?;

            state.mailer.send(&email)?;

            // reply with OK
            let res = Response::builder(StatusCode::Ok).build();
            Ok(res)
        }
        Err(form_errors) => {
            let res = Response::builder(StatusCode::BadRequest)
                .body(json!(form_errors))
                .build();
            Ok(res)
        }
    }
}

/// Endpoint for `POST /register/verify`
pub(crate) async fn verify(mut req: Request<State>) -> Result<Response> {
    let form: VerifyForm = req.body_form().await?;

    match form.validate() {
        Ok(()) => {
            // acquire state
            let state = req.state();
            let key = form.key;
            let full_key = format!("{}{}", REGISTER_KEY_PREFIX, &key);

            let encoded_reg_info: Vec<u8> = state.redis_manager.clone().get(&full_key).await?;

            // if it's empty, then the registration likely expired
            if encoded_reg_info.is_empty() {
                let res = Response::builder(StatusCode::BadRequest).build();
                return Ok(res);
            }

            // insert a new user into the database with this info
            let reg_info: RegistrationInfo = bincode::deserialize(&encoded_reg_info)?;

            let mut db_conn = state.db_pool.acquire().await?;

            sqlx::query_as!(
                User,
                r#"INSERT INTO users (name, email, password) VALUES ($1, $2, $3)"#,
                reg_info.name,
                reg_info.email,
                reg_info.password
            )
            .execute(&mut db_conn)
            .await?;

            // delete value at key
            state.redis_manager.clone().del(&full_key).await?;

            let res = Response::builder(StatusCode::Ok).build();
            Ok(res)
        }
        Err(form_errors) => {
            let res = Response::builder(StatusCode::BadRequest)
                .body(json!(form_errors))
                .build();
            Ok(res)
        }
    }
}
