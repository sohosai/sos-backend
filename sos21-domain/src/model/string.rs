use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use thiserror::Error;

use crate::model::bound::{Bound, Bounded};

/// A length-limited string.
///
/// This provides a wrapper to validate that the string's length is
/// between `Lower` and `Upper` bounds for all `AsRef<str>` types.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LengthLimitedString<Lower, Upper, S> {
    _lower: PhantomData<Lower>,
    _upper: PhantomData<Upper>,
    len: usize,
    inner: S,
}

pub type LengthBoundedString<Min, Max, S> = LengthLimitedString<Bounded<Min>, Bounded<Max>, S>;

impl<Lower, Upper, S> LengthLimitedString<Lower, Upper, S> {
    pub fn new(s: S) -> Result<Self, LengthError<Lower, Upper>>
    where
        S: AsRef<str>,
        Lower: Bound<usize>,
        Upper: Bound<usize>,
    {
        let len = s.as_ref().chars().count();
        if let Some(lower) = Lower::limit() {
            if len < lower {
                return Err(LengthError {
                    kind: LengthErrorKind::TooShort,
                    _upper: PhantomData,
                    _lower: PhantomData,
                });
            }
        }
        if let Some(upper) = Upper::limit() {
            if len > upper {
                return Err(LengthError {
                    kind: LengthErrorKind::TooLong,
                    _upper: PhantomData,
                    _lower: PhantomData,
                });
            }
        }

        Ok(LengthLimitedString {
            _upper: PhantomData,
            _lower: PhantomData,
            len,
            inner: s,
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthErrorKind {
    TooLong,
    TooShort,
}

pub type BoundedLengthError<Min, Max> = LengthError<Bounded<Min>, Bounded<Max>>;

#[derive(Error, Clone)]
#[error("the string's length is out of bounds")]
pub struct LengthError<Lower, Upper> {
    kind: LengthErrorKind,
    _lower: PhantomData<Lower>,
    _upper: PhantomData<Upper>,
}

impl<Lower, Upper> Debug for LengthError<Lower, Upper> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LengthError")
            .field("kind", &self.kind)
            .finish()
    }
}

impl<Lower, Upper> LengthError<Lower, Upper> {
    pub fn kind(&self) -> LengthErrorKind {
        self.kind
    }
}

// This cannot be generic because of the blanket impl `TryFrom<T> for U where U: Into<T>`
impl<Lower, Upper> TryFrom<String> for LengthLimitedString<Lower, Upper, String>
where
    Lower: Bound<usize>,
    Upper: Bound<usize>,
{
    type Error = LengthError<Lower, Upper>;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        LengthLimitedString::new(s)
    }
}

// This cannot be generic because of the orphan rule
impl<Lower, Upper> From<LengthLimitedString<Lower, Upper, String>> for String {
    fn from(s: LengthLimitedString<Lower, Upper, String>) -> String {
        s.into_inner()
    }
}

impl<Lower, Upper, S: AsRef<str>> AsRef<str> for LengthLimitedString<Lower, Upper, S> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl<Lower, Upper, S: Display> Display for LengthLimitedString<Lower, Upper, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <S as Display>::fmt(&self.inner, f)
    }
}

impl<Lower, Upper, S: Debug> Debug for LengthLimitedString<Lower, Upper, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <S as Debug>::fmt(&self.inner, f)
    }
}

impl<Lower, Upper> FromStr for LengthLimitedString<Lower, Upper, String>
where
    Lower: Bound<usize>,
    Upper: Bound<usize>,
{
    type Err = LengthError<Lower, Upper>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_owned();
        LengthLimitedString::new(s)
    }
}

impl<Lower, Upper, S> Serialize for LengthLimitedString<Lower, Upper, S>
where
    S: Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, Lower, Upper, S> Deserialize<'de> for LengthLimitedString<Lower, Upper, S>
where
    Lower: Bound<usize>,
    Upper: Bound<usize>,
    S: Deserialize<'de> + AsRef<str>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        LengthLimitedString::new(S::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// A kana (either hiragana or katakana) string.
///
/// This provides a wrapper to validate that the string is kana for all `AsRef<str>` types.
/// The accept characters are:
///
/// - Hiragana: U+3040 ..= U+309F
/// - Katakana: U+30A0 ..= U+30FF
/// - Half-width katakana: U+FF65 ..= U+FF9F
/// - and characters which has `White_Space` property, including half-width and full-width spaces.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KanaString<S>(S);

impl<S> KanaString<S> {
    pub fn new(s: S) -> Result<Self, NotKanaError>
    where
        S: AsRef<str>,
    {
        if is_kana_string(s.as_ref()) {
            Ok(KanaString(s))
        } else {
            Err(NotKanaError { _priv: () })
        }
    }

    pub fn into_inner(self) -> S {
        self.0
    }
}

fn is_kana_char(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{309F}' |
        '\u{30A0}'..='\u{30FF}' |
        '\u{FF65}'..='\u{FF9F}'
    ) || c.is_whitespace()
}

fn is_kana_string(s: &str) -> bool {
    s.chars().all(is_kana_char)
}

#[derive(Debug, Error, Clone)]
#[error("invalid kana string")]
pub struct NotKanaError {
    _priv: (),
}

// This cannot be generic because of the blanket impl `TryFrom<T> for U where U: Into<T>`
impl TryFrom<String> for KanaString<String> {
    type Error = NotKanaError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        KanaString::new(s)
    }
}

// This cannot be generic because of the orphan rule
impl From<KanaString<String>> for String {
    fn from(s: KanaString<String>) -> String {
        s.into_inner()
    }
}

impl<S: AsRef<str>> AsRef<str> for KanaString<S> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<S: Display> Display for KanaString<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <S as Display>::fmt(&self.0, f)
    }
}

impl<S: Debug> Debug for KanaString<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <S as Debug>::fmt(&self.0, f)
    }
}

impl FromStr for KanaString<String> {
    type Err = NotKanaError;
    fn from_str(s: &str) -> Result<KanaString<String>, Self::Err> {
        let s = s.to_owned();
        KanaString::new(s)
    }
}

/// A stripped string that does not have whitespace on both ends.
///
/// This provides a wrapper to validate that the string is stripped for all `AsRef<str>` types.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StrippedString<S>(S);

impl<S> StrippedString<S>
where
    S: AsRef<str>,
{
    pub fn new(s: S) -> Result<Self, NotStrippedError> {
        if is_stripped_string(s.as_ref()) {
            Ok(StrippedString(s))
        } else {
            Err(NotStrippedError { _priv: () })
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl<S> StrippedString<S> {
    pub fn into_inner(self) -> S {
        self.0
    }
}

fn is_stripped_string(s: &str) -> bool {
    !s.starts_with(char::is_whitespace) && !s.ends_with(char::is_whitespace)
}

#[derive(Debug, Error, Clone)]
#[error("not stripped string")]
pub struct NotStrippedError {
    _priv: (),
}

// This cannot be generic because of the blanket impl `TryFrom<T> for U where U: Into<T>`
impl TryFrom<String> for StrippedString<String> {
    type Error = NotStrippedError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        StrippedString::new(s)
    }
}

// This cannot be generic because of the orphan rule
impl From<StrippedString<String>> for String {
    fn from(s: StrippedString<String>) -> String {
        s.into_inner()
    }
}

impl<S: AsRef<str>> AsRef<str> for StrippedString<S> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<S: Display> Display for StrippedString<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <S as Display>::fmt(&self.0, f)
    }
}

impl<S: Debug> Debug for StrippedString<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <S as Debug>::fmt(&self.0, f)
    }
}

impl FromStr for StrippedString<String> {
    type Err = NotStrippedError;
    fn from_str(s: &str) -> Result<StrippedString<String>, Self::Err> {
        let s = s.to_owned();
        StrippedString::new(s)
    }
}

impl<S> Serialize for StrippedString<S>
where
    S: Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, S> Deserialize<'de> for StrippedString<S>
where
    S: Deserialize<'de> + AsRef<str>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        StrippedString::new(S::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{KanaString, LengthLimitedString, StrippedString};
    use crate::model::bound::{Bounded, Unbounded};
    use std::str::FromStr;

    #[test]
    fn test_not_kana() {
        assert!(KanaString::from_str("計算機").is_err());
        assert!(KanaString::from_str("雙峰祭 オンラインシステム").is_err());
    }

    #[test]
    fn test_kana_kana() {
        assert!(KanaString::from_str("コンピュータ").is_ok());
        assert!(KanaString::from_str("オンライン　システム").is_ok());
        assert!(KanaString::from_str("そう なんだ").is_ok());
    }

    type NonEmptyString<S> = LengthLimitedString<Bounded<typenum::U1>, Unbounded, S>;

    #[test]
    fn test_nonempty_empty() {
        assert!(NonEmptyString::from_str("").is_err());
    }

    #[test]
    fn test_nonempty_nonempty() {
        assert!(NonEmptyString::from_str("あ").is_ok(),);
        assert!(NonEmptyString::from_str("あbc").is_ok(),);
    }

    #[test]
    fn test_bounded() {
        assert!(
            LengthLimitedString::<Unbounded, Bounded<typenum::U3>, String>::from_str("").is_ok()
        );
        assert!(
            LengthLimitedString::<Unbounded, Bounded<typenum::U3>, String>::from_str("abc").is_ok()
        );
        assert!(
            LengthLimitedString::<Unbounded, Bounded<typenum::U3>, String>::from_str("abcd")
                .is_err()
        );
        assert!(
            LengthLimitedString::<Unbounded, Bounded<typenum::U2>, String>::from_str("計算機")
                .is_err()
        );
        assert!(
            LengthLimitedString::<Bounded<typenum::U1>, Bounded<typenum::U3>, String>::from_str(
                "a"
            )
            .is_ok()
        );
        assert!(
            LengthLimitedString::<Bounded<typenum::U1>, Bounded<typenum::U3>, String>::from_str("")
                .is_err()
        );
        assert!(
            LengthLimitedString::<Bounded<typenum::U2>, Bounded<typenum::U3>, String>::from_str(
                "あ"
            )
            .is_err()
        );
    }

    #[test]
    fn test_stripped() {
        assert!(StrippedString::from_str("").is_ok());
        assert!(StrippedString::from_str("abc").is_ok());
        assert!(StrippedString::from_str("a b c").is_ok());
        assert!(StrippedString::from_str("雙峰祭 オンラインシステム").is_ok());
    }

    #[test]
    fn test_not_stripped() {
        assert!(StrippedString::from_str(" ").is_err());
        assert!(StrippedString::from_str("　").is_err());
        assert!(StrippedString::from_str(" abc").is_err());
        assert!(StrippedString::from_str("abc　").is_err());
        assert!(StrippedString::from_str("a b c ").is_err());
        assert!(StrippedString::from_str("　雙峰祭 オンラインシステム").is_err());
        assert!(StrippedString::from_str("　雙峰祭 ").is_err());
    }
}
