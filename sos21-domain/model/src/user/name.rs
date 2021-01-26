use crate::string::{KanaString, LengthBoundedString};
use crate::user::User;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserName {
    first: LengthBoundedString<typenum::U1, typenum::U64, String>,
    last: LengthBoundedString<typenum::U1, typenum::U64, String>,
}

#[derive(Debug, Error, Clone)]
#[error("invalid user name")]
pub struct NameError {
    _priv: (),
}

impl UserName {
    pub fn from_string(
        first: impl Into<String>,
        last: impl Into<String>,
    ) -> Result<Self, NameError> {
        let first = LengthBoundedString::new(first.into()).map_err(|_| NameError { _priv: () })?;
        let last = LengthBoundedString::new(last.into()).map_err(|_| NameError { _priv: () })?;
        Ok(UserName { first, last })
    }

    pub fn first(&self) -> &str {
        self.first.as_ref()
    }

    pub fn last(&self) -> &str {
        self.last.as_ref()
    }

    pub fn into_string(self) -> (String, String) {
        (self.first.into_inner(), self.last.into_inner())
    }

    pub fn is_visible_to(&self, _user: &User) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserKanaName {
    first: KanaString<LengthBoundedString<typenum::U1, typenum::U256, String>>,
    last: KanaString<LengthBoundedString<typenum::U1, typenum::U256, String>>,
}

#[derive(Debug, Error, Clone)]
#[error("invalid user kana name")]
pub struct KanaNameError {
    _priv: (),
}

impl UserKanaName {
    pub fn from_string(
        first: impl Into<String>,
        last: impl Into<String>,
    ) -> Result<Self, KanaNameError> {
        let first =
            LengthBoundedString::new(first.into()).map_err(|_| KanaNameError { _priv: () })?;
        let first = KanaString::new(first).map_err(|_| KanaNameError { _priv: () })?;
        let last =
            LengthBoundedString::new(last.into()).map_err(|_| KanaNameError { _priv: () })?;
        let last = KanaString::new(last).map_err(|_| KanaNameError { _priv: () })?;
        Ok(UserKanaName { first, last })
    }

    pub fn first(&self) -> &str {
        self.first.as_ref()
    }

    pub fn last(&self) -> &str {
        self.last.as_ref()
    }

    pub fn into_string(self) -> (String, String) {
        (
            self.first.into_inner().into_inner(),
            self.last.into_inner().into_inner(),
        )
    }
}

#[cfg(test)]
mod tests {
    use sos21_domain_test::model as test_model;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let name = test_model::mock_user_name();
        assert!(name.is_visible_to(&user));
    }
}
