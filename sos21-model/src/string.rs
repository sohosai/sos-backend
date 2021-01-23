use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Display, Write};
use std::str::FromStr;

use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use thiserror::Error;

/// Correct by construction non-empty string.
///
/// [`Debug`][debug], [`Display`][display], and [`Serialize`][serialize] implementations behave just
/// like [`String`][string].
///
/// [debug]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
/// [display]: https://doc.rust-lang.org/std/fmt/trait.Display.html
/// [serialize]: https://docs.serde.rs/serde/trait.Serialize.html
/// [string]: https://doc.rust-lang.org/std/string/struct.String.html
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyString {
    pub init: String,
    pub last: char,
}

impl From<NonEmptyString> for String {
    fn from(NonEmptyString { mut init, last }: NonEmptyString) -> String {
        init.push(last);
        init
    }
}

#[derive(Debug, Error, Clone)]
#[error("invalid empty string")]
pub struct EmptyError {
    _priv: (),
}

impl TryFrom<String> for NonEmptyString {
    type Error = EmptyError;
    fn try_from(mut s: String) -> Result<NonEmptyString, Self::Error> {
        if let Some(last) = s.pop() {
            Ok(NonEmptyString { init: s, last })
        } else {
            Err(EmptyError { _priv: () })
        }
    }
}

impl Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.init)?;
        f.write_char(self.last)
    }
}

impl Debug for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char('"')?;
        f.write_str(&self.init)?;
        f.write_char(self.last)?;
        f.write_char('"')
    }
}

impl FromStr for NonEmptyString {
    type Err = EmptyError;
    fn from_str(s: &str) -> Result<NonEmptyString, Self::Err> {
        let s = s.to_owned();
        s.try_into()
    }
}

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .try_into()
            .map_err(de::Error::custom)
    }
}

impl Serialize for NonEmptyString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::NonEmptyString;
    use std::str::FromStr;

    #[test]
    fn test_nonempty_string_empty() {
        assert!(NonEmptyString::from_str("").is_err());
    }

    #[test]
    fn test_nonempty_string_nonempty() {
        assert_eq!(
            NonEmptyString::from_str("あ").unwrap(),
            NonEmptyString {
                init: String::new(),
                last: 'あ'
            }
        );
        assert_eq!(
            NonEmptyString::from_str("あbc").unwrap(),
            NonEmptyString {
                init: "あb".to_string(),
                last: 'c'
            }
        );
    }
}
