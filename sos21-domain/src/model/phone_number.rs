use thiserror::Error;

/// A valid phone number which consists of ~15 digit numbers prefixed with '+' and the country code.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhoneNumber(String);

#[derive(Debug, Error, Clone)]
#[error("invalid phone number")]
pub struct FromStringError {
    _priv: (),
}

impl PhoneNumber {
    pub fn from_string(phone_number: impl Into<String>) -> Result<Self, FromStringError> {
        let phone_number = phone_number.into();

        if !is_valid_phone_number(&phone_number) {
            return Err(FromStringError { _priv: () });
        }

        Ok(PhoneNumber(phone_number))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

// TODO: Check country code validity
fn is_valid_phone_number(s: &str) -> bool {
    let rest = match s.strip_prefix('+') {
        Some(x) => x,
        None => return false,
    };

    if rest.len() > 15 || rest.len() < 2 {
        return false;
    }

    if rest.bytes().any(|c| !c.is_ascii_digit()) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::PhoneNumber;

    #[test]
    fn test_phone_number_invalid() {
        assert!(PhoneNumber::from_string("").is_err());
    }

    #[test]
    fn test_phone_number_valid() {
        assert!(PhoneNumber::from_string("+81300000000").is_ok());
        assert!(PhoneNumber::from_string("+819000000000").is_ok());
    }
}
