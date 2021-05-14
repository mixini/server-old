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

const REGISTER_EXPIRY_SECONDS: usize = 86400;

#[derive(Debug, Validate, Deserialize)]
/// The form of a `POST /register` request.
struct RegisterForm {
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
    /// The provided username.
    pub(crate) username: String,
    #[validate(email(message = "Must be a valid email address."))]
    /// The provided email.
    pub(crate) email: String,
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
    /// The provided password.
    pub(crate) password: String,
}

/// A local, intermediate structure for storing registration info, distinct from `RegisterForm` in that the password is hashed, so the plaintext password is never stored.
#[derive(Debug, Serialize, Deserialize)]
struct RegistrationInfo {
    /// The username.
    username: String,
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
    reg_key: String,
}

/// Endpoint for `POST /register`
pub(crate) async fn register(mut req: Request<State>) -> Result<Response> {
    let form: RegisterForm = req.body_form().await?;

    match form.validate() {
        Ok(()) => {
            // acquire state
            let state = req.state();

            // record this registration in redis
            let reg_key: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();

            // hash the password
            let hashed_pw = match HASHER.hash(&form.password) {
                Ok(s) => s,
                Err(err) => {
                    tide::log::error!("Error occurred with hasher: {:?}", err);
                    let res = Response::builder(StatusCode::InternalServerError).build();
                    return Ok(res);
                }
            };

            // encode this registration info
            let reg_info = RegistrationInfo {
                username: form.username,
                email: form.email,
                password: hashed_pw,
            };
            let encoded_reg_info: Vec<u8> = bincode::serialize(&reg_info)?;

            state
                .redis_manager
                .clone()
                .set_ex(
                    reg_key.to_owned(),
                    encoded_reg_info,
                    REGISTER_EXPIRY_SECONDS,
                )
                .await?;

            // send an email to the registrant with a code to verify
            let email = Message::builder()
                .from(std::env::var("SMTP_EMAIL")?.parse()?)
                .to(reg_info.email.parse()?)
                .subject("Your Mixini registration verification")
                .body(format!(
                    "Your Mixini registration key is {}. Note that it will expire in 24 hours.",
                    reg_key
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

            let encoded_reg_info: Vec<u8> = state.redis_manager.clone().get(form.reg_key).await?;

            // if it's empty, then the registration likely expired
            if encoded_reg_info.is_empty() {
                let res = Response::builder(StatusCode::BadRequest).build();
                return Ok(res);
            }

            // TODO: insert a new user into the database with this info
            let reg_info = bincode::deserialize(&encoded_reg_info)?;

            unimplemented!()
        }
        Err(form_errors) => {
            let res = Response::builder(StatusCode::BadRequest)
                .body(json!(form_errors))
                .build();
            Ok(res)
        }
    }
}
