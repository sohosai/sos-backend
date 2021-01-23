use std::fmt::{self, Debug, Display};
use std::str::FromStr;

use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmailAddress(String);

#[derive(Debug, Error, Clone)]
#[error("invalid email address")]
pub struct FromStringError {
    _priv: (),
}

impl EmailAddress {
    pub fn from_string(s: String) -> Result<EmailAddress, FromStringError> {
        if is_valid_email_address(&s) {
            Ok(EmailAddress(s))
        } else {
            Err(FromStringError { _priv: () })
        }
    }
}

impl From<EmailAddress> for String {
    fn from(email: EmailAddress) -> String {
        email.0
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for EmailAddress {
    type Err = FromStringError;
    fn from_str(s: &str) -> Result<EmailAddress, Self::Err> {
        let s = s.to_owned();
        EmailAddress::from_string(s)
    }
}

impl<'de> Deserialize<'de> for EmailAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        EmailAddress::from_string(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

impl Serialize for EmailAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
fn is_valid_email_address(s: &str) -> bool {
    let mut sp = s.splitn(2, '@');
    let local_part = match sp.next() {
        None => return false,
        Some(x) if x.is_empty() => return false,
        Some(x) => x,
    };
    let domain = match sp.next() {
        None => return false,
        Some(x) if x.is_empty() => return false,
        Some(x) => x,
    };
    assert!(sp.next().is_none());

    if !local_part.bytes().all(|c| {
        c == b'.' || c.is_ascii_alphanumeric() || (c.is_ascii_punctuation() && !is_specials(c))
    }) {
        return false;
    }

    for label in domain.split('.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        if label.starts_with('-') || label.ends_with('-') {
            return false;
        }
        if !label
            .bytes()
            .all(|c| c == b'-' || c.is_ascii_alphanumeric())
        {
            return false;
        }
    }
    return true;
}

fn is_specials(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b':' | b';' | b'@' | b'\\' | b',' | b'.' | b'"'
    )
}

#[cfg(test)]
mod tests {
    use super::EmailAddress;
    use std::str::FromStr;

    #[test]
    fn test_address_invalid() {
        assert!(EmailAddress::from_str("").is_err());
        assert!(EmailAddress::from_str("a@a@a").is_err());
        assert!(EmailAddress::from_str("a(b)c@a.b").is_err());
        assert!(EmailAddress::from_str("ab@a-.b").is_err());
        assert!(EmailAddress::from_str("@a.b").is_err());
        assert!(EmailAddress::from_str("a@").is_err());
    }

    #[test]
    fn test_address_valid() {
        assert!(EmailAddress::from_str("a@b.c").is_ok());
        assert!(EmailAddress::from_str("a.b.c@de.fg").is_ok());
        assert!(EmailAddress::from_str("ab.c@d-e.fg").is_ok());
    }
}
