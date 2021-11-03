use sqlx::types::chrono::Utc;
use uuid::Uuid;

/// User model
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) created_at: Utc,
    pub(crate) updated_at: Utc,
    pub(crate) name: String,
    /// The password in hashed PHC form, as represented in the database
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) verified: bool,
}
