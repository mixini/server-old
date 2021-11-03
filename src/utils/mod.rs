//! Miscellaneous utils
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub(crate) mod pass;

/// Generate random key for use in Redis given the prefix
pub(crate) fn generate_redis_key(prefix: &str) -> String {
    let key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    format!("{}{}", prefix, &key)
}
