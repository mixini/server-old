use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// User roles
#[derive(Debug, PartialEq, sqlx::Type)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
pub(crate) enum Role {
    Admin,
    Moderator,
    Maintainer,
    Creator,
    Contributor,
    Member,
}

/// User model
#[derive(Debug, sqlx::FromRow, oso::PolarClass)]
pub(crate) struct User {
    #[polar(attribute)]
    pub(crate) id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) role: Role,
    /// The password in hashed PHC form, as represented in the database
    pub(crate) password: String,
    #[polar(attribute)]
    pub(crate) verified: bool,
}
