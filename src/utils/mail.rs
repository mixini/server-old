//! Handles email stuff.

use lazy_static::lazy_static;
use lettre::{
    message::Mailbox, transport::smtp::response::Response, AsyncSmtpTransport, AsyncTransport,
    Message, Tokio1Executor,
};

use crate::error::MixiniError;

lazy_static! {
    static ref SMTP_EMAIL: Mailbox = std::env::var("SMTP_EMAIL")
        .expect("SMTP_EMAIL is not set in env")
        .parse()
        .expect("SMTP_EMAIL key is invalid");
}

pub async fn send_email_verification_request(
    mailsender: &AsyncSmtpTransport<Tokio1Executor>,
    email: String,
    key: String,
) -> Result<Response, MixiniError> {
    let email = email.parse().expect("somehow not verified?");
    let mail = Message::builder()
        .from(SMTP_EMAIL.to_owned())
        .to(email)
        .subject("Your Mixini email verification")
        .body(format!(
            "Your Mixini verification key is {}. Note that it will expire in 24 hours.",
            key
        ))?;

    Ok(mailsender.send(mail).await?)
}
