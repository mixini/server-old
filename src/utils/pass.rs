//! Password-related utilities.

use lazy_static::lazy_static;
use libreauth::pass::{Algorithm, HashBuilder, Hasher};

pub(crate) const PWD_ALGORITHM: Algorithm = Algorithm::Argon2;
pub(crate) const PWD_SCHEME_VERSION: usize = 1;

// If the Hasher changes, make sure to increment PWD_SCHEME_VERSION
lazy_static! {
    pub(crate) static ref HASHER: Hasher = {
        HashBuilder::new()
            .algorithm(PWD_ALGORITHM)
            .version(PWD_SCHEME_VERSION)
            .finalize()
            .unwrap()
    };
}
