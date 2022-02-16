use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use std::fmt;
use uuid::Uuid;

use crate::{handlers::UpdateUserForm, impl_redis_rv};

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
    #[polar(attribute)]
    pub(crate) name: String,
    #[polar(attribute)]
    pub(crate) email: String,
    #[polar(attribute)]
    pub(crate) role: Role,
    /// The password in hashed PHC form, as represented in the database
    #[polar(attribute)]
    pub(crate) password: String,
    #[polar(attribute)]
    pub(crate) verified: bool,
}

/// User model but all fields are options
#[derive(Debug, Clone, oso::PolarClass, Serialize, Deserialize)]
pub(crate) struct UserOptional {
    #[polar(attribute)]
    pub(crate) name: Option<String>,
    pub(crate) created_at: Option<DateTime<Utc>>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
    #[polar(attribute)]
    pub(crate) email: Option<String>,
    #[polar(attribute)]
    pub(crate) role: Option<Role>,
    #[polar(attribute)]
    pub(crate) password: Option<String>,
    pub(crate) verified: Option<bool>,
}

impl From<UpdateUserForm> for UserOptional {
    fn from(form: UpdateUserForm) -> Self {
        Self {
            name: form.name,
            email: form.email,
            created_at: None,
            updated_at: None,
            role: form.role,
            password: form.password,
            verified: None,
        }
    }
}

impl_redis_rv!(User, Role);
