use std::fmt::{self, Debug};
use std::marker::PhantomData;

use crate::model::string::{self, KanaString, LengthBoundedString, StrippedString};

use num_rational::Ratio;
use thiserror::Error;
use typenum::Unsigned;
use unicode_segmentation::UnicodeSegmentation;

enum NameLengthError {
    TooLong,
    Empty,
}

/// A project-name string.
///
/// Project-name strings has a maximum length limit (that is parameterized by Max).
/// The spec have unique length counting scheme for the project name strings.
/// We basically count the number of graphemes clusters in the string,
/// but half-width graphic characters and both half-width and full-width alphanumeric characters are
/// counted as 2/3 full-width kana character.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ProjectNameString<Max> {
    _max: PhantomData<Max>,
    inner: String,
}

impl<Max> Debug for ProjectNameString<Max> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <String as Debug>::fmt(&self.inner, f)
    }
}

impl<Max> AsRef<str> for ProjectNameString<Max> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<Max> ProjectNameString<Max> {
    pub fn from_string(s: String) -> Result<Self, NameLengthError>
    where
        Max: Unsigned,
    {
        fn is_small(c: char) -> bool {
            matches!(c,
                '\u{FF10}' ..= '\u{FF19}' |
                '\u{FF21}' ..= '\u{FF3A}' |
                '\u{FF41}' ..= '\u{FF5A}'
            ) || c.is_ascii_graphic()
        }

        let len: Ratio<u64> = s
            .graphemes(true)
            .map(|grapheme_cluster| {
                let mut chars = grapheme_cluster.chars();
                if let Some(scalar) = chars.next() {
                    if is_small(scalar) && chars.next().is_none() {
                        return Ratio::new(2, 3);
                    }
                }

                Ratio::from_integer(1)
            })
            .sum();

        if *len.numer() == 0 {
            return Err(NameLengthError::Empty);
        }

        if len > Ratio::from_integer(Max::to_u64()) {
            return Err(NameLengthError::TooLong);
        }

        Ok(ProjectNameString {
            _max: PhantomData,
            inner: s,
        })
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_ref()
    }

    pub fn into_string(self) -> String {
        self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectName(StrippedString<ProjectNameString<typenum::U22>>);

#[derive(Debug, Error, Clone)]
#[error("invalid project name")]
pub struct NameError {
    _priv: (),
}

impl NameError {
    fn from_length_error(_err: NameLengthError) -> Self {
        NameError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        NameError { _priv: () }
    }
}

impl ProjectName {
    pub fn from_string(name: impl Into<String>) -> Result<Self, NameError> {
        let inner =
            ProjectNameString::from_string(name.into()).map_err(NameError::from_length_error)?;
        let inner = StrippedString::new(inner).map_err(NameError::from_not_stripped_error)?;
        Ok(ProjectName(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectKanaName(
    StrippedString<KanaString<LengthBoundedString<typenum::U1, typenum::U128, String>>>,
);

#[derive(Debug, Error, Clone)]
#[error("invalid project kana name")]
pub struct KanaNameError {
    _priv: (),
}

impl KanaNameError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U128>) -> Self {
        KanaNameError { _priv: () }
    }

    fn from_not_kana_error(_err: string::NotKanaError) -> Self {
        KanaNameError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        KanaNameError { _priv: () }
    }
}

impl ProjectKanaName {
    pub fn from_string(kana_name: impl Into<String>) -> Result<Self, KanaNameError> {
        let inner =
            LengthBoundedString::new(kana_name.into()).map_err(KanaNameError::from_length_error)?;
        let inner = KanaString::new(inner).map_err(KanaNameError::from_not_kana_error)?;
        let inner = StrippedString::new(inner).map_err(KanaNameError::from_not_stripped_error)?;
        Ok(ProjectKanaName(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner().into_inner()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGroupName(StrippedString<ProjectNameString<typenum::U25>>);

#[derive(Debug, Error, Clone)]
#[error("invalid project group name")]
pub struct GroupNameError {
    _priv: (),
}

impl GroupNameError {
    fn from_length_error(_err: NameLengthError) -> Self {
        GroupNameError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        GroupNameError { _priv: () }
    }
}

impl ProjectGroupName {
    pub fn from_string(kana_name: impl Into<String>) -> Result<Self, GroupNameError> {
        let inner = ProjectNameString::from_string(kana_name.into())
            .map_err(GroupNameError::from_length_error)?;
        let inner = StrippedString::new(inner).map_err(GroupNameError::from_not_stripped_error)?;
        Ok(ProjectGroupName(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_string()
    }
}

#[derive(Debug, Error, Clone)]
#[error("invalid project kana group name")]
pub struct KanaGroupNameError {
    _priv: (),
}

impl KanaGroupNameError {
    fn from_length_error(_err: string::BoundedLengthError<typenum::U1, typenum::U128>) -> Self {
        KanaGroupNameError { _priv: () }
    }

    fn from_not_kana_error(_err: string::NotKanaError) -> Self {
        KanaGroupNameError { _priv: () }
    }

    fn from_not_stripped_error(_err: string::NotStrippedError) -> Self {
        KanaGroupNameError { _priv: () }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectKanaGroupName(
    StrippedString<KanaString<LengthBoundedString<typenum::U1, typenum::U128, String>>>,
);

impl ProjectKanaGroupName {
    pub fn from_string(kana_name: impl Into<String>) -> Result<Self, KanaGroupNameError> {
        let inner = LengthBoundedString::new(kana_name.into())
            .map_err(KanaGroupNameError::from_length_error)?;
        let inner = KanaString::new(inner).map_err(KanaGroupNameError::from_not_kana_error)?;
        let inner =
            StrippedString::new(inner).map_err(KanaGroupNameError::from_not_stripped_error)?;
        Ok(ProjectKanaGroupName(inner))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0.into_inner().into_inner().into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::{GroupNameError, NameError, ProjectGroupName, ProjectName};

    #[test]
    fn test_name_empty() {
        assert!(matches!(
            ProjectName::from_string(""),
            Err(NameError { .. })
        ));
        assert!(matches!(
            ProjectName::from_string("   "),
            Err(NameError { .. })
        ));
    }

    #[test]
    fn test_name_too_long() {
        assert!(matches!(
            ProjectName::from_string("a".repeat(34)),
            Err(NameError { .. })
        ));
        assert!(matches!(
            ProjectName::from_string("あ".repeat(23)),
            Err(NameError { .. })
        ));
        assert!(matches!(
            ProjectName::from_string(format!("a{}", "あ".repeat(22))),
            Err(NameError { .. })
        ));
    }

    #[test]
    fn test_name_pass() {
        assert_eq!(ProjectName::from_string("!").unwrap().into_string(), "!");
        assert_eq!(ProjectName::from_string("ok").unwrap().into_string(), "ok");
        let a33 = "a".repeat(33);
        assert_eq!(ProjectName::from_string(&a33).unwrap().into_string(), a33);
        assert_eq!(
            ProjectName::from_string("こんにちは")
                .unwrap()
                .into_string(),
            "こんにちは"
        );
        let kana22 = "あ".repeat(22);
        assert_eq!(
            ProjectName::from_string(&kana22).unwrap().into_string(),
            kana22
        );
        let mixed = format!("a{}", "あ".repeat(21));
        assert_eq!(
            ProjectName::from_string(&mixed).unwrap().into_string(),
            mixed
        );
    }

    #[test]
    fn test_group_name_empty() {
        assert!(matches!(
            ProjectGroupName::from_string(""),
            Err(GroupNameError { .. })
        ));
        assert!(matches!(
            ProjectGroupName::from_string("   "),
            Err(GroupNameError { .. })
        ));
    }

    #[test]
    fn test_group_name_too_long() {
        assert!(matches!(
            ProjectGroupName::from_string("a".repeat(38)),
            Err(GroupNameError { .. })
        ));
        assert!(matches!(
            ProjectGroupName::from_string("あ".repeat(26)),
            Err(GroupNameError { .. })
        ));
        assert!(matches!(
            ProjectGroupName::from_string(format!("a{}", "あ".repeat(25))),
            Err(GroupNameError { .. })
        ));
    }

    #[test]
    fn test_group_name_pass() {
        assert_eq!(
            ProjectGroupName::from_string("!").unwrap().into_string(),
            "!"
        );
        assert_eq!(
            ProjectGroupName::from_string("ok").unwrap().into_string(),
            "ok"
        );
        let a37 = "a".repeat(37);
        assert_eq!(
            ProjectGroupName::from_string(&a37).unwrap().into_string(),
            a37
        );
        assert_eq!(
            ProjectGroupName::from_string("こんにちは")
                .unwrap()
                .into_string(),
            "こんにちは"
        );
        let kana25 = "あ".repeat(25);
        assert_eq!(
            ProjectGroupName::from_string(&kana25)
                .unwrap()
                .into_string(),
            kana25
        );
        let mixed = format!("a{}", "あ".repeat(24));
        assert_eq!(
            ProjectGroupName::from_string(&mixed).unwrap().into_string(),
            mixed
        );
    }
}
