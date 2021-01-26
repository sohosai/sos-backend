use chrono::Utc;
use sos21_domain_model::{
    email::EmailAddress,
    user::{User, UserId, UserKanaName, UserName, UserRole},
};
use uuid::Uuid;

pub fn new_user_id() -> UserId {
    UserId(Uuid::new_v4().to_string())
}

pub fn mock_user_name() -> UserName {
    UserName::from_string("太郎", "ユーザー").unwrap()
}

pub fn mock_user_kana_name() -> UserKanaName {
    UserKanaName::from_string("タロウ", "ユーザー").unwrap()
}

pub fn mock_email_address() -> EmailAddress {
    EmailAddress::from_string("hello@example.com".to_string()).unwrap()
}

pub fn new_user(role: UserRole) -> User {
    User {
        id: new_user_id(),
        created_at: Utc::now(),
        name: mock_user_name(),
        kana_name: mock_user_kana_name(),
        email: mock_email_address(),
        role,
    }
}

pub fn new_general_user() -> User {
    new_user(UserRole::General)
}

pub fn new_committee_user() -> User {
    new_user(UserRole::Committee)
}

pub fn new_operator_user() -> User {
    new_user(UserRole::CommitteeOperator)
}

pub fn new_admin_user() -> User {
    new_user(UserRole::Administrator)
}
