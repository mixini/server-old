use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use tide::{convert::json, Request, Response, Result};
use validator::Validate;

use crate::server::State;

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9\.\-_]+$").unwrap();
    static ref RE_PASSWORD: Regex = Regex::new(r"^[a-zA-Z0-9]*[0-9][a-zA-Z0-9]*$").unwrap();
}

#[derive(Debug, Validate, Deserialize)]
pub(crate) struct RegisterForm {
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
    username: String,
    #[validate(email(message = "Must be a valid email address."))]
    email: String,
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

/// Endpoint for `POST /register`
pub(crate) async fn register(mut req: Request<State>) -> Result<Response> {
    let form: RegisterForm = req.body_form().await?;

    match form.validate() {
        Ok(form) => {
            // TODO: record this registration in redis

            // TODO: send an email to the registrant with a code to verify

            // reply with OK
            let res = Response::builder(200).build();
            Ok(res)
        }
        Err(form_errors) => {
            let res = Response::builder(400).body(json!(form_errors)).build();
            Ok(res)
        }
    }
}
