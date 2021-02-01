use std::fmt::{self, Debug, Display};
use std::str::FromStr;

use crate::model::string::{Bounded, LengthLimitedString};

use thiserror::Error;

/// A valid email address whose length is ~128 chars.
///
/// The email is validated against the definition used in the living HTML standard.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmailAddress(LengthLimitedString<Bounded<typenum::U3>, Bounded<typenum::U128>, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid email address")]
pub struct FromStringError {
    _priv: (),
}

impl EmailAddress {
    pub fn from_string(s: String) -> Result<EmailAddress, FromStringError> {
        if is_valid_email_address(&s) {
            if let Ok(s) = LengthLimitedString::new(s) {
                return Ok(EmailAddress(s));
            }
        }

        Err(FromStringError { _priv: () })
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }

    pub fn ends_with(&self, suffix: impl AsRef<str>) -> bool {
        self.0.as_ref().ends_with(suffix.as_ref())
    }
}

impl From<EmailAddress> for String {
    fn from(email: EmailAddress) -> String {
        email.0.into()
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for EmailAddress {
    type Err = FromStringError;
    fn from_str(s: &str) -> Result<EmailAddress, Self::Err> {
        let s = s.to_owned();
        EmailAddress::from_string(s)
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
    debug_assert!(sp.next().is_none());

    if !local_part.bytes().all(|c| c == b'.' || is_atext(c)) {
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
    true
}

// https://tools.ietf.org/html/rfc5322#section-3.2.3
fn is_atext(c: u8) -> bool {
    c.is_ascii_alphanumeric() || (c.is_ascii_punctuation() && !is_specials(c))
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
