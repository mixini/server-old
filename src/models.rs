//! Database models
//!
//! Note that these may have to be updated by hand.

use sqlx::types::chrono::Utc;
use uuid::Uuid;

/// User model
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) created_at: Utc,
    pub(crate) updated_at: Utc,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) password: String,
}
