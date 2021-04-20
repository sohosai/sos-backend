use crate::model::string::{self, KanaString, LengthBoundedString, StrippedString};
use crate::model::user::User;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserName {
    first: StrippedString<LengthBoundedString<typenum::U1, typenum::U64, String>>,
    last: StrippedString<LengthBoundedString<typenum::U1, typenum::U64, String>>,
}

#[derive(Debug, Error, Clone)]
#[error("invalid user name")]
pub struct NameError {
    _priv: (),
}

impl NameError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U64>) -> Self {
        NameError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        NameError { _priv: () }
    }
}

impl UserName {
    pub fn from_string(
        first: impl Into<String>,
        last: impl Into<String>,
    ) -> Result<Self, NameError> {
        let first = LengthBoundedString::new(first.into()).map_err(NameError::from_length_error)?;
        let first = StrippedString::new(first).map_err(NameError::from_not_stripped_error)?;
        let last = LengthBoundedString::new(last.into()).map_err(NameError::from_length_error)?;
        let last = StrippedString::new(last).map_err(NameError::from_not_stripped_error)?;
        Ok(UserName { first, last })
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

    pub fn is_visible_to(&self, _user: &User) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserKanaName {
    first: StrippedString<KanaString<LengthBoundedString<typenum::U1, typenum::U256, String>>>,
    last: StrippedString<KanaString<LengthBoundedString<typenum::U1, typenum::U256, String>>>,
}

#[derive(Debug, Error, Clone)]
#[error("invalid user kana name")]
pub struct KanaNameError {
    _priv: (),
}

impl KanaNameError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U256>) -> Self {
        KanaNameError { _priv: () }
    }

    fn from_not_kana_error(_err: string::NotKanaError) -> Self {
        KanaNameError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        KanaNameError { _priv: () }
    }
}

impl UserKanaName {
    pub fn from_string(
        first: impl Into<String>,
        last: impl Into<String>,
    ) -> Result<Self, KanaNameError> {
        let first =
            LengthBoundedString::new(first.into()).map_err(KanaNameError::from_length_error)?;
        let first = KanaString::new(first).map_err(KanaNameError::from_not_kana_error)?;
        let first = StrippedString::new(first).map_err(KanaNameError::from_not_stripped_error)?;
        let last =
            LengthBoundedString::new(last.into()).map_err(KanaNameError::from_length_error)?;
        let last = KanaString::new(last).map_err(KanaNameError::from_not_kana_error)?;
        let last = StrippedString::new(last).map_err(KanaNameError::from_not_stripped_error)?;
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
            self.first.into_inner().into_inner().into_inner(),
            self.last.into_inner().into_inner().into_inner(),
        )
    }

    pub fn is_visible_to(&self, _user: &User) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let name = test_model::mock_user_name();
        assert!(name.is_visible_to(&user));
    }
}
