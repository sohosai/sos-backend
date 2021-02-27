use crate::context::ProjectRepository;
use crate::model::{
    string::{self, LengthBoundedString},
    user::User,
};

use thiserror::Error;

/// A project's display ID consists of lowercase alphanumeric or '_' characters whose length is 3~64.
///
/// Note that the project ID won't start with '_'.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDisplayId(LengthBoundedString<typenum::U3, typenum::U64, String>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayIdErrorKind {
    TooLong,
    TooShort,
    ContainsDisallowedCharacter,
    StartsWithUnderscore,
}

impl DisplayIdErrorKind {
    fn from_length_error_kind(kind: string::LengthErrorKind) -> Self {
        match kind {
            string::LengthErrorKind::TooShort => DisplayIdErrorKind::TooShort,
            string::LengthErrorKind::TooLong => DisplayIdErrorKind::TooLong,
        }
    }
}

#[derive(Debug, Error, Clone)]
#[error("invalid project display id")]
pub struct DisplayIdError {
    kind: DisplayIdErrorKind,
}

impl DisplayIdError {
    fn from_length_error(err: string::BoundedLengthError<typenum::U3, typenum::U64>) -> Self {
        DisplayIdError {
            kind: DisplayIdErrorKind::from_length_error_kind(err.kind()),
        }
    }
}

impl DisplayIdError {
    pub fn kind(&self) -> DisplayIdErrorKind {
        self.kind
    }
}

impl ProjectDisplayId {
    pub fn from_string(display_id: impl Into<String>) -> Result<Self, DisplayIdError> {
        let display_id = LengthBoundedString::new(display_id.into())
            .map_err(DisplayIdError::from_length_error)?;

        validate_display_id(display_id.as_ref())?;
        Ok(ProjectDisplayId(display_id))
    }

    pub fn into_string(self) -> String {
        self.0.into_inner()
    }

    pub async fn is_available<C>(&self, ctx: C) -> anyhow::Result<bool>
    where
        C: ProjectRepository,
    {
        ctx.find_project_by_display_id(self.clone())
            .await
            .map(|opt| opt.is_none())
    }

    // Display ID itself is visible to every user, though projects cannot be seen by others.
    // This is because we need to do a check for every user that some display ID is available or not.
    pub fn is_visible_to(&self, _user: &User) -> bool {
        true
    }
}

fn validate_display_id(s: &str) -> Result<(), DisplayIdError> {
    if s.starts_with('_') {
        return Err(DisplayIdError {
            kind: DisplayIdErrorKind::StartsWithUnderscore,
        });
    }

    if !s.bytes().all(is_valid_display_id_character) {
        return Err(DisplayIdError {
            kind: DisplayIdErrorKind::ContainsDisallowedCharacter,
        });
    }

    Ok(())
}

fn is_valid_display_id_character(c: u8) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_'
}

#[cfg(test)]
mod tests {
    use super::{DisplayIdErrorKind, ProjectDisplayId};

    #[test]
    fn test_display_id_invalid() {
        assert_eq!(
            ProjectDisplayId::from_string("").unwrap_err().kind(),
            DisplayIdErrorKind::TooShort
        );
        assert_eq!(
            ProjectDisplayId::from_string("ac").unwrap_err().kind(),
            DisplayIdErrorKind::TooShort
        );
        assert_eq!(
            ProjectDisplayId::from_string("a@bc").unwrap_err().kind(),
            DisplayIdErrorKind::ContainsDisallowedCharacter
        );
        assert_eq!(
            ProjectDisplayId::from_string("a(b)c").unwrap_err().kind(),
            DisplayIdErrorKind::ContainsDisallowedCharacter
        );
        assert_eq!(
            ProjectDisplayId::from_string("a-bc").unwrap_err().kind(),
            DisplayIdErrorKind::ContainsDisallowedCharacter
        );
        assert_eq!(
            ProjectDisplayId::from_string("_abc").unwrap_err().kind(),
            DisplayIdErrorKind::StartsWithUnderscore
        );
        assert!(ProjectDisplayId::from_string("a-").is_err());
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

    #[tokio::test]
    async fn test_unavailable() {
        use crate::test;

        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build();

        assert!(!project.display_id.is_available(&app).await.unwrap());
    }

    #[tokio::test]
    async fn test_available() {
        use crate::test;

        let user = test::model::new_general_user();
        let project_id = test::model::new_project_display_id();
        let app = test::build_mock_app().users(vec![user.clone()]).build();

        assert!(project_id.is_available(&app).await.unwrap());
    }
}
