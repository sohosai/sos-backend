use std::fmt::{self, Display, Write};
use std::str::{self, FromStr};

use crate::model::project::{index, ProjectIndex};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectKind {
    pub is_cooking: bool,
    pub is_outdoor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectCode {
    pub kind: ProjectKind,
    pub index: ProjectIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseCodeErrorKind {
    MismatchedLength,
    UnknownCookingFlag,
    UnknownOutdoorFlag,
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
        if s.len() != 5 {
            return Err(ParseCodeError {
                kind: ParseCodeErrorKind::MismatchedLength,
            });
        }

        let is_cooking = match s[0] {
            b'C' => true,
            b'G' => false,
            _ => {
                return Err(ParseCodeError {
                    kind: ParseCodeErrorKind::UnknownCookingFlag,
                })
            }
        };
        let is_outdoor = match s[1] {
            b'O' => true,
            b'I' => false,
            _ => {
                return Err(ParseCodeError {
                    kind: ParseCodeErrorKind::UnknownOutdoorFlag,
                })
            }
        };
        let kind = ProjectKind {
            is_cooking,
            is_outdoor,
        };

        let index = str::from_utf8(&s[2..5]).map_err(ParseCodeError::from_index_utf8_error)?;
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
        let cooking = if self.kind.is_cooking { 'C' } else { 'G' };
        let outdoor = if self.kind.is_outdoor { 'O' } else { 'I' };
        f.write_char(cooking)?;
        f.write_char(outdoor)?;
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
            ProjectCode::parse("GI001").unwrap(),
            ProjectCode {
                kind: ProjectKind {
                    is_cooking: false,
                    is_outdoor: false,
                },
                index: ProjectIndex::from_u16(1).unwrap(),
            }
        );

        assert_eq!(
            ProjectCode::parse("CO000").unwrap(),
            ProjectCode {
                kind: ProjectKind {
                    is_cooking: true,
                    is_outdoor: true,
                },
                index: ProjectIndex::from_u16(0).unwrap(),
            }
        );

        assert_eq!(
            ProjectCode::parse("CI999").unwrap(),
            ProjectCode {
                kind: ProjectKind {
                    is_cooking: true,
                    is_outdoor: false,
                },
                index: ProjectIndex::from_u16(999).unwrap(),
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
            ProjectCode::parse("CO01").unwrap_err().kind(),
            ParseCodeErrorKind::MismatchedLength
        );
        assert_eq!(
            ProjectCode::parse("COFFF").unwrap_err().kind(),
            ParseCodeErrorKind::InvalidIndex
        );
        assert!(ProjectCode::parse("XA000").is_err());
        assert_eq!(
            ProjectCode::parse("AI001").unwrap_err().kind(),
            ParseCodeErrorKind::UnknownCookingFlag
        );
        assert_eq!(
            ProjectCode::parse("CA001").unwrap_err().kind(),
            ParseCodeErrorKind::UnknownOutdoorFlag
        );
    }

    #[test]
    fn test_serialize_deserialize() {
        fn ser_de(code: ProjectCode) {
            assert_eq!(ProjectCode::parse(&code.to_string()).unwrap(), code);
        }

        ser_de(ProjectCode {
            kind: ProjectKind {
                is_cooking: true,
                is_outdoor: true,
            },
            index: ProjectIndex::from_u16(0).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind {
                is_cooking: false,
                is_outdoor: true,
            },
            index: ProjectIndex::from_u16(125).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind {
                is_cooking: true,
                is_outdoor: false,
            },
            index: ProjectIndex::from_u16(999).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind {
                is_cooking: false,
                is_outdoor: false,
            },
            index: ProjectIndex::from_u16(10).unwrap(),
        });
    }

    #[test]
    fn test_deserialize_serialize() {
        fn de_ser(code: &str) {
            assert_eq!(ProjectCode::parse(code).unwrap().to_string(), code);
        }

        de_ser("CO000");
        de_ser("CI001");
        de_ser("GO101");
        de_ser("GI010");
        de_ser("CO999");
    }
}
