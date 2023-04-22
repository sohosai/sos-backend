use std::fmt::{self, Display, Write};
use std::str::{self, FromStr};

use crate::model::project::{index, ProjectCategory, ProjectIndex};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    General,
    CookingRequiringPreparationArea,
    Cooking,
    Food,
    Stage,
}

impl From<ProjectCategory> for ProjectKind {
    fn from(from: ProjectCategory) -> Self {
        match from {
            ProjectCategory::General => Self::General,
            ProjectCategory::CookingRequiringPreparationArea => Self::CookingRequiringPreparationArea,
            ProjectCategory::Cooking => Self::Cooking,
            ProjectCategory::Food => Self::Food,
            ProjectCategory::Stage => Self::Stage,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectCode {
    pub kind: ProjectKind,
    pub index: ProjectIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseCodeErrorKind {
    MismatchedLength,
    UnknownGroup,
    InvalidIndex,
}

#[derive(Debug, Error, Clone)]
#[error("invalid project code")]
pub struct ParseCodeError {
    kind: ParseCodeErrorKind,
}

impl ParseCodeError {
    fn from_index_utf8_error(_err: str::Utf8Error) -> Self {
        ParseCodeError {
            kind: ParseCodeErrorKind::InvalidIndex,
        }
    }

    fn from_index_parse_error(_err: std::num::ParseIntError) -> Self {
        ParseCodeError {
            kind: ParseCodeErrorKind::InvalidIndex,
        }
    }

    fn from_index_error(_err: index::FromU16Error) -> Self {
        ParseCodeError {
            kind: ParseCodeErrorKind::InvalidIndex,
        }
    }

    pub fn kind(&self) -> ParseCodeErrorKind {
        self.kind
    }
}

impl ProjectCode {
    pub fn parse(s: &str) -> Result<Self, ParseCodeError> {
        ProjectCode::parse_bytes(s.as_bytes())
    }

    pub fn parse_bytes(s: &[u8]) -> Result<Self, ParseCodeError> {
        let kind = match s.len() {
            4 => match s[0] {
                b'G' => ProjectKind::General,
                b'S' => ProjectKind::Stage,
                b'C' => ProjectKind::Cooking,
                b'A' => ProjectKind::CookingRequiringPreparationArea,
                b'F' => ProjectKind::Food,
                _ => {
                    return Err(ParseCodeError {
                        kind: ParseCodeErrorKind::UnknownGroup,
                    })
                }
            },
            _ => {
                return Err(ParseCodeError {
                    kind: ParseCodeErrorKind::MismatchedLength,
                })
            }
        };

        let index_slice = match s.len() {
            4 => &s[1..4],
            5 => &s[2..5],
            _ => unreachable!(),
        };

        let index = str::from_utf8(index_slice).map_err(ParseCodeError::from_index_utf8_error)?;
        let index = index
            .parse()
            .map_err(ParseCodeError::from_index_parse_error)?;
        let index = ProjectIndex::from_u16(index).map_err(ParseCodeError::from_index_error)?;
        Ok(ProjectCode { kind, index })
    }
}

impl FromStr for ProjectCode {
    type Err = ParseCodeError;
    fn from_str(s: &str) -> Result<ProjectCode, Self::Err> {
        ProjectCode::parse(s)
    }
}

impl Display for ProjectCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let group = match self.kind {
            ProjectKind::General => 'G',
            ProjectKind::Stage => 'S',
            ProjectKind::Food => 'F',
            ProjectKind::Cooking => 'C',
            ProjectKind::CookingRequiringPreparationArea => 'A'
        };

        f.write_char(group)?;
        write!(f, "{:03}", self.index.to_u16())
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseCodeErrorKind, ProjectCode, ProjectKind};
    use crate::model::project::ProjectIndex;

    #[test]
    fn test_valid() {
        assert_eq!(
            ProjectCode::parse("G001").unwrap(),
            ProjectCode {
                kind: ProjectKind::General,
                index: ProjectIndex::from_u16(1).unwrap(),
            }
        );

        assert_eq!(
            ProjectCode::parse("S000").unwrap(),
            ProjectCode {
                kind: ProjectKind::Stage,
                index: ProjectIndex::from_u16(0).unwrap(),
            }
        );

        assert_eq!(
            ProjectCode::parse("F000").unwrap(),
            ProjectCode {
                kind: ProjectKind::Food,
                index: ProjectIndex::from_u16(0).unwrap(),
            }
        );

        assert_eq!(
            ProjectCode::parse("C000").unwrap(),
            ProjectCode {
                kind: ProjectKind::Cooking,
                index: ProjectIndex::from_u16(0).unwrap(),
            }
        );

        assert_eq!(
            ProjectCode::parse("A000").unwrap(),
            ProjectCode {
                kind: ProjectKind::CookingRequiringPreparationArea,
                index: ProjectIndex::from_u16(0).unwrap(),
            }
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(
            ProjectCode::parse("").unwrap_err().kind(),
            ParseCodeErrorKind::MismatchedLength
        );

        assert_eq!(
            ProjectCode::parse("G0001").unwrap_err().kind(),
            ParseCodeErrorKind::MismatchedLength
        );
        assert_eq!(
            ProjectCode::parse("OGFFF").unwrap_err().kind(),
            ParseCodeErrorKind::MismatchedLength
        );
        assert!(ProjectCode::parse("XA000").is_err());

        assert_eq!(
            ProjectCode::parse("B001").unwrap_err().kind(),
            ParseCodeErrorKind::UnknownGroup
        );
    }

    #[test]
    fn test_serialize_deserialize() {
        fn ser_de(code: ProjectCode) {
            assert_eq!(ProjectCode::parse(&code.to_string()).unwrap(), code);
        }

        ser_de(ProjectCode {
            kind: ProjectKind::General,
            index: ProjectIndex::from_u16(1).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind::Stage,
            index: ProjectIndex::from_u16(0).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind::Food,
            index: ProjectIndex::from_u16(0).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind::Cooking,
            index: ProjectIndex::from_u16(0).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind::CookingRequiringPreparationArea,
            index: ProjectIndex::from_u16(0).unwrap(),
        });
    }

    #[test]
    fn test_deserialize_serialize() {
        fn de_ser(code: &str) {
            assert_eq!(ProjectCode::parse(code).unwrap().to_string(), code);
        }

        de_ser("G000");
        de_ser("S001");
        de_ser("C010");
        de_ser("A999");
        de_ser("F011");
    }
}
