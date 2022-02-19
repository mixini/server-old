//! Constants

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub(crate) static ref DOMAIN: String =
        std::env::var("DOMAIN").expect("DOMAIN is not set in env");
    pub(crate) static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9\.\-_]+$").unwrap();
    pub(crate) static ref RE_PASSWORD: Regex =
        Regex::new(r"^[a-zA-Z0-9]*[0-9][a-zA-Z0-9]*$").unwrap();
}

// for authorized sessions
pub(crate) const SESSION_COOKIE_NAME: &str = "msessid";
pub(crate) const SESSION_KEY_PREFIX: &str = "session:";
pub(crate) const SESSION_DURATION_SECS: usize = 1209600;

// for user verify requests
pub(crate) const VERIFY_KEY_PREFIX: &str = "verify:";
pub(crate) const VERIFY_EXPIRY_SECONDS: usize = 86400;
