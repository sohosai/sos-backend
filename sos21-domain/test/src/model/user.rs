use sos21_domain_model::{
    date_time::DateTime,
    phone_number::PhoneNumber,
    user::{User, UserAffiliation, UserEmailAddress, UserId, UserKanaName, UserName, UserRole},
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

pub fn mock_user_email_address() -> UserEmailAddress {
    UserEmailAddress::from_string("example@s.tsukuba.ac.jp").unwrap()
}

pub fn mock_phone_number() -> PhoneNumber {
    PhoneNumber::from_string("+81900000000").unwrap()
}

pub fn mock_user_affiliation() -> UserAffiliation {
    UserAffiliation::from_string("情報学群情報科学類").unwrap()
}

pub fn new_user(role: UserRole) -> User {
    User {
        id: new_user_id(),
        created_at: DateTime::now(),
        name: mock_user_name(),
        kana_name: mock_user_kana_name(),
        email: mock_user_email_address(),
        phone_number: mock_phone_number(),
        affiliation: mock_user_affiliation(),
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
