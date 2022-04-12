use crate::handler::model::date_time::DateTime;

use serde::{Deserialize, Serialize};
use sos21_use_case::model::user as use_case;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub String);

impl UserId {
    pub fn from_use_case(id: use_case::UserId) -> UserId {
        UserId(id.0)
    }

    pub fn into_use_case(self) -> use_case::UserId {
        use_case::UserId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserName {
    pub first: String,
    pub last: String,
}

impl UserName {
    pub fn from_use_case(name: use_case::UserName) -> UserName {
        UserName {
            first: name.first,
            last: name.last,
        }
    }

    pub fn into_use_case(self) -> use_case::UserName {
        use_case::UserName {
            first: self.first,
            last: self.last,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKanaName {
    pub first: String,
    pub last: String,
}

impl UserKanaName {
    pub fn from_use_case(name: use_case::UserKanaName) -> UserKanaName {
        UserKanaName {
            first: name.first,
            last: name.last,
        }
    }

    pub fn into_use_case(self) -> use_case::UserKanaName {
        use_case::UserKanaName {
            first: self.first,
            last: self.last,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Administrator,
    CommitteeOperator,
    Committee,
    General,
}

impl UserRole {
    pub fn from_use_case(role: use_case::UserRole) -> UserRole {
        match role {
            use_case::UserRole::Administrator => UserRole::Administrator,
            use_case::UserRole::CommitteeOperator => UserRole::CommitteeOperator,
            use_case::UserRole::Committee => UserRole::Committee,
            use_case::UserRole::General => UserRole::General,
        }
    }

    pub fn into_use_case(self) -> use_case::UserRole {
        match self {
            UserRole::Administrator => use_case::UserRole::Administrator,
            UserRole::CommitteeOperator => use_case::UserRole::CommitteeOperator,
            UserRole::Committee => use_case::UserRole::Committee,
            UserRole::General => use_case::UserRole::General,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserCategory {
    UndergraduateStudent,
    GraduateStudent,
    AcademicStaff,
}

impl UserCategory {
    pub fn from_use_case(category: use_case::UserCategory) -> UserCategory {
        match category {
            use_case::UserCategory::UndergraduateStudent => UserCategory::UndergraduateStudent,
            use_case::UserCategory::GraduateStudent => UserCategory::GraduateStudent,
            use_case::UserCategory::AcademicStaff => UserCategory::AcademicStaff,
        }
    }

    pub fn into_use_case(self) -> use_case::UserCategory {
        match self {
            UserCategory::UndergraduateStudent=>use_case::UserCategory::UndergraduateStudent,
            UserCategory::GraduateStudent => use_case::UserCategory::GraduateStudent,
            UserCategory::AcademicStaff => use_case::UserCategory::AcademicStaff,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub created_at: DateTime,
    pub name: UserName,
    pub kana_name: UserKanaName,
    pub email: String,
    pub phone_number: String,
    pub role: UserRole,
    pub category: UserCategory,
}

impl User {
    pub fn from_use_case(user: use_case::User) -> User {
        User {
            id: UserId::from_use_case(user.id),
            created_at: DateTime::from_use_case(user.created_at),
            name: UserName::from_use_case(user.name),
            kana_name: UserKanaName::from_use_case(user.kana_name),
            email: user.email,
            phone_number: user.phone_number,
            role: UserRole::from_use_case(user.role),
            category: UserCategory::from_use_case(user.category),
        }
    }
}
