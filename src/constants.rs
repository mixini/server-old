//! Constants

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref DOMAIN: String = std::env::var("DOMAIN").expect("DOMAIN is not set in env");
    pub static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9\.\-_]+$").unwrap();
    pub static ref RE_PASSWORD: Regex = Regex::new(r"^[a-zA-Z0-9]*[0-9][a-zA-Z0-9]*$").unwrap();
}

// for authorized sessions
pub const SESSION_COOKIE_NAME: &str = "msessid";
pub const SESSION_KEY_PREFIX: &str = "session:";
pub const SESSION_DURATION_SECS: usize = 1209600;

// for user verify requests
pub const VERIFY_KEY_PREFIX: &str = "verify:";
pub const VERIFY_EXPIRY_SECONDS: usize = 86400;
