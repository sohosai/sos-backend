use crate::email::EmailAddress;
use crate::role::Role;
use crate::string::NonEmptyString;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub String);

impl From<UserId> for String {
    fn from(id: UserId) -> String {
        id.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserName {
    pub first: NonEmptyString,
    pub last: NonEmptyString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub created_at: DateTime<Utc>,
    pub name: UserName,
    pub email: EmailAddress,
    pub role: Role,
}
