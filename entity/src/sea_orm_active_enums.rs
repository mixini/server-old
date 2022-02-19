//! SeaORM Entity. Generated by sea-orm-codegen 0.6.0

use oso::PolarClass;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, PolarClass,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
pub enum UserRole {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "contributor")]
    Contributor,
    #[sea_orm(string_value = "creator")]
    Creator,
    #[sea_orm(string_value = "maintainer")]
    Maintainer,
    #[sea_orm(string_value = "member")]
    Member,
    #[sea_orm(string_value = "moderator")]
    Moderator,
}