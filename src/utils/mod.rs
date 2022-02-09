//! Miscellaneous utils
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub(crate) mod mail;
pub(crate) mod pass;

const KEY_LENGTH: usize = 32;

/// Pair of keys intended for use in redis and cookies
pub(crate) struct RKeys {
    /// Key without prefix
    pub(crate) base_key: String,
    /// Key with prefix
    pub(crate) prefixed_key: String,
}

impl RKeys {
    /// Generate a random alphanumeric key `KEY_LENGTH` long and return its' `(raw, prefixed)` variations.
    pub(crate) fn generate(prefix: &'static str) -> Self {
        let base_key = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(KEY_LENGTH)
            .map(char::from)
            .collect();
        let prefixed_key = format!("{}{}", prefix, base_key);
        Self {
            base_key,
            prefixed_key,
        }
    }
}
