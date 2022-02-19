//! CRUD action-like resources
use anyhow::Result;
use entity::sea_orm_active_enums::UserRole;
use oso::{Oso, PolarClass};
use serde::Deserialize;
use validator::Validate;

use crate::constants::RE_USERNAME;

/// The "READ" action. Because there is no data pertinent to this action it is a unit struct.
#[derive(Debug, Clone, Copy, PolarClass)]
pub struct Read;

/// The "DELETE" action. Because there is no data pertinent to this action it is a unit struct.
#[derive(Debug, Clone, Copy, PolarClass)]
pub struct Delete;

/// The action by which a user is updated. Can be understood as a sort of changeset.
///
/// This struct in particular doubles up for multiple use cases. It's used for PUT `/user/:id` form responses,
/// in authorization rules, and also for updates to the ORM.
#[derive(Debug, Clone, Validate, Deserialize, PolarClass)]
pub struct UpdateUser {
    #[validate(
        length(
            min = 5,
            max = 32,
            message = "Minimum length is 5 characters, maximum is 32"
        ),
        regex(
            path = "RE_USERNAME",
            message = "Can only contain letters, numbers, dashes (-), periods (.), and underscores (_)"
        )
    )]
    #[polar(attribute)]
    pub name: Option<String>,
    #[validate(email(message = "Must be a valid email address."))]
    #[polar(attribute)]
    pub email: Option<String>,
    #[polar(attribute)]
    pub role: Option<UserRole>,
}

/// Attempt to create a new oso instance for managing authorization schemes.
pub fn try_register_oso() -> Result<Oso> {
    let mut oso = Oso::new();

    // NOTE: load classes here
    oso.register_class(entity::user_account::Model::get_polar_class())?;
    oso.register_class(UserRole::get_polar_class())?;

    // action classes in this module should be loaded here too
    oso.register_class(Read::get_polar_class())?;
    oso.register_class(Delete::get_polar_class())?;
    oso.register_class(UpdateUser::get_polar_class())?;

    // NOTE: load oso rule files here
    oso.load_files(vec!["polar/users.polar"])?;

    Ok(oso)
}
