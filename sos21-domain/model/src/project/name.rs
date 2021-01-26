use crate::string::{KanaString, LengthBoundedString};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectName(LengthBoundedString<typenum::U1, typenum::U128, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid project name")]
pub struct NameError {
    _priv: (),
}

impl ProjectName {
    pub fn from_string(name: impl Into<String>) -> Result<Self, NameError> {
        let inner = LengthBoundedString::new(name.into()).map_err(|_| NameError { _priv: () })?;
        Ok(ProjectName(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectKanaName(KanaString<LengthBoundedString<typenum::U1, typenum::U512, String>>);

#[derive(Debug, Error, Clone)]
#[error("invalid project kana name")]
pub struct KanaNameError {
    _priv: (),
}

impl ProjectKanaName {
    pub fn from_string(kana_name: impl Into<String>) -> Result<Self, KanaNameError> {
        let inner =
            LengthBoundedString::new(kana_name.into()).map_err(|_| KanaNameError { _priv: () })?;
        let inner = KanaString::new(inner).map_err(|_| KanaNameError { _priv: () })?;
        Ok(ProjectKanaName(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGroupName(LengthBoundedString<typenum::U1, typenum::U128, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid project group name")]
pub struct GroupNameError {
    _priv: (),
}

impl ProjectGroupName {
    pub fn from_string(kana_name: impl Into<String>) -> Result<Self, GroupNameError> {
        let inner =
            LengthBoundedString::new(kana_name.into()).map_err(|_| GroupNameError { _priv: () })?;
        Ok(ProjectGroupName(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}

#[derive(Debug, Error, Clone)]
#[error("invalid project kana group name")]
pub struct KanaGroupNameError {
    _priv: (),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectKanaGroupName(
    KanaString<LengthBoundedString<typenum::U1, typenum::U512, String>>,
);

impl ProjectKanaGroupName {
    pub fn from_string(kana_name: impl Into<String>) -> Result<Self, KanaGroupNameError> {
        let inner = LengthBoundedString::new(kana_name.into())
            .map_err(|_| KanaGroupNameError { _priv: () })?;
        let inner = KanaString::new(inner).map_err(|_| KanaGroupNameError { _priv: () })?;
        Ok(ProjectKanaGroupName(inner))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner()
    }
}
