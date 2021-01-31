use crate::string::LengthBoundedString;

use thiserror::Error;

/// A project's display ID consists of lowercase alphanumeric or '_' characters whose length is 3~64.
///
/// Note that the project ID won't start with '_'.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDisplayId(LengthBoundedString<typenum::U3, typenum::U64, String>);

#[derive(Debug, Error, Clone)]
#[error("invalid project display id")]
pub struct DisplayIdError {
    _priv: (),
}

impl ProjectDisplayId {
    pub fn from_string(display_id: impl Into<String>) -> Result<Self, DisplayIdError> {
        let display_id = LengthBoundedString::new(display_id.into())
            .map_err(|_| DisplayIdError { _priv: () })?;
        if !is_valid_display_id(display_id.as_ref()) {
            return Err(DisplayIdError { _priv: () });
        }

        Ok(ProjectDisplayId(display_id))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}

fn is_valid_display_id(s: &str) -> bool {
    if s.starts_with('_') {
        return false;
    }

    s.bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

#[cfg(test)]
mod tests {
    use super::ProjectDisplayId;

    #[test]
    fn test_display_id_invalid() {
        assert!(ProjectDisplayId::from_string("").is_err());
        assert!(ProjectDisplayId::from_string("ac").is_err());
        assert!(ProjectDisplayId::from_string("a@bc").is_err());
        assert!(ProjectDisplayId::from_string("a(b)c").is_err());
        assert!(ProjectDisplayId::from_string("a-bc").is_err());
        assert!(ProjectDisplayId::from_string("_abc").is_err());
        assert!(ProjectDisplayId::from_string("___").is_err());
    }

    #[test]
    fn test_display_id_valid() {
        assert!(ProjectDisplayId::from_string("abc").is_ok());
        assert!(ProjectDisplayId::from_string("120").is_ok());
        assert!(ProjectDisplayId::from_string("a_b1d_e").is_ok());
        assert!(ProjectDisplayId::from_string("ac_").is_ok());
        assert!(ProjectDisplayId::from_string("a__c").is_ok());
    }
}
