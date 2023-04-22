use std::convert::TryInto;
use std::fmt::{self, Display};

use crate::model::user::User;

use chrono::{DateTime, Utc};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "project_category")]
#[sqlx(rename_all = "snake_case")]
pub enum ProjectCategory {
    General, // 一般企画（飲食物取扱い企画、調理企画を除く）
    CookingRequiringPreparationArea, // 一般企画（調理企画（仕込場が必要））
    Cooking, // 一般企画（調理企画（仕込場が不要））
    Food, // 一般企画（飲食物取扱い企画）
    Stage, // ステージ企画
}

bitflags::bitflags! {
    pub struct ProjectAttributes: u32 {
        const ACADEMIC  = 0b00000001;
        const ARTISTIC  = 0b00000010;
        const COMMITTEE = 0b00000100;
        const OUTDOOR   = 0b00001000;
    }
}

impl sqlx::Type<sqlx::Postgres> for ProjectAttributes {
    fn type_info() -> PgTypeInfo {
        <i32 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ProjectAttributes {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> sqlx::encode::IsNull {
        <i32 as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&(self.bits() as i32), buf)
    }
}

#[derive(Debug)]
struct FromBitsError;

impl Display for FromBitsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid project attributes")
    }
}

impl std::error::Error for FromBitsError {}

impl sqlx::Decode<'_, sqlx::Postgres> for ProjectAttributes {
    fn decode(value: PgValueRef) -> Result<Self, sqlx::error::BoxDynError> {
        let bits = <i32 as sqlx::Decode<sqlx::Postgres>>::decode(value)?.try_into()?;
        ProjectAttributes::from_bits(bits)
            .ok_or_else::<sqlx::error::BoxDynError, _>(|| Box::new(FromBitsError))
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub index: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

#[derive(Debug, Clone)]
pub struct ProjectWithOwners {
    pub project: Project,
    pub owner: User,
    pub subowner: User,
}
