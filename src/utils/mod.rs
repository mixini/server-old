//! Miscellaneous utils
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub(crate) mod mail;
pub(crate) mod pass;

const KEY_LENGTH: usize = 32;

/// Generate random key for use in Redis given a prefix
pub(crate) fn generate_redis_key(prefix: &'static str) -> String {
    let key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(KEY_LENGTH)
        .map(char::from)
        .collect();
    format!("{}{}", prefix, &key)
}
