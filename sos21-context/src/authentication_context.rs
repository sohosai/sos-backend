use sos21_model::{email::EmailAddress, user::UserId};

pub trait AuthenticationContext {
    fn login_user(&self) -> UserId;
    fn login_email(&self) -> EmailAddress;
}
