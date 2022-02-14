use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use std::fmt;
use uuid::Uuid;

use crate::impl_redis_rv;

/// User roles
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, sqlx::Type, oso::PolarClass, Serialize, Deserialize,
)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
pub(crate) enum Role {
    Admin,
    Moderator,
    Maintainer,
    Creator,
    Contributor,
    Member,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{:?}", self).to_ascii_lowercase();
        write!(f, "{}", s)
    }
}

/// User model
#[derive(Debug, Clone, sqlx::FromRow, oso::PolarClass, Serialize, Deserialize)]
pub(crate) struct User {
    #[polar(attribute)]
    pub(crate) id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) name: String,
    pub(crate) email: String,
    #[polar(attribute)]
    pub(crate) role: Role,
    /// The password in hashed PHC form, as represented in the database
    pub(crate) password: String,
    #[polar(attribute)]
    pub(crate) verified: bool,
}

impl_redis_rv!(User, Role);
