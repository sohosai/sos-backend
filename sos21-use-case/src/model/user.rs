use chrono::{DateTime, Utc};
use sos21_domain_model::user as entity;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(pub String);

impl UserId {
    pub fn from_entity(id: entity::UserId) -> UserId {
        UserId(id.0)
    }

    pub fn into_entity(self) -> entity::UserId {
        entity::UserId(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserName {
    pub first: String,
    pub last: String,
}

impl UserName {
    pub fn from_entity(name: entity::UserName) -> UserName {
        let (first, last) = name.into_string();
        UserName { first, last }
    }

    pub fn into_entity(self) -> Option<entity::UserName> {
        entity::UserName::from_string(self.first, self.last).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserKanaName {
    pub first: String,
    pub last: String,
}

impl UserKanaName {
    pub fn from_entity(name: entity::UserKanaName) -> UserKanaName {
        let (first, last) = name.into_string();
        UserKanaName { first, last }
    }

    pub fn into_entity(self) -> Option<entity::UserKanaName> {
        entity::UserKanaName::from_string(self.first, self.last).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserRole {
    Administrator,
    CommitteeOperator,
    Committee,
    General,
}

impl UserRole {
    pub fn from_entity(role: entity::UserRole) -> UserRole {
        match role {
            entity::UserRole::Administrator => UserRole::Administrator,
            entity::UserRole::CommitteeOperator => UserRole::CommitteeOperator,
            entity::UserRole::Committee => UserRole::Committee,
            entity::UserRole::General => UserRole::General,
        }
    }

    pub fn into_entity(self) -> entity::UserRole {
        match self {
            UserRole::Administrator => entity::UserRole::Administrator,
            UserRole::CommitteeOperator => entity::UserRole::CommitteeOperator,
            UserRole::Committee => entity::UserRole::Committee,
            UserRole::General => entity::UserRole::General,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub created_at: DateTime<Utc>,
    pub name: UserName,
    pub kana_name: UserKanaName,
    pub email: String,
    pub role: UserRole,
}

impl User {
    pub fn from_entity(user: entity::User) -> User {
        User {
            id: UserId::from_entity(user.id),
            created_at: user.created_at,
            name: UserName::from_entity(user.name),
            kana_name: UserKanaName::from_entity(user.kana_name),
            email: user.email.into_string(),
            role: UserRole::from_entity(user.role),
        }
    }
}
