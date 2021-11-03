use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// User model
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) name: String,
    /// The password in hashed PHC form, as represented in the database
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) verified: bool,
}
