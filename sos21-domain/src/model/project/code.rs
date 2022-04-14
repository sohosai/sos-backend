use std::fmt::{self, Display, Write};
use std::str::{self, FromStr};

use crate::model::project::{index, ProjectCategory, ProjectIndex};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    General { is_online: bool },
    Stage { is_online: bool },
    Cooking,
    Food,
}

impl From<ProjectCategory> for ProjectKind {
    fn from(from: ProjectCategory) -> Self {
        match from {
            ProjectCategory::GeneralOnline => Self::General { is_online: true },
            ProjectCategory::GeneralPhysical => Self::General { is_online: false },
            ProjectCategory::StageOnline => Self::Stage { is_online: true },
            ProjectCategory::StagePhysical => Self::Stage { is_online: false },
            ProjectCategory::CookingPhysical => Self::Cooking,
            ProjectCategory::FoodPhysical => Self::Food,
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
    UnknownOnlineFlag,
    UnknownGroup,
    MissingOnlineFlag,
    GotExtraOnlineFlag,
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
                b'G' | b'S' => {
                    return Err(ParseCodeError {
                        kind: ParseCodeErrorKind::MissingOnlineFlag,
                    })
                }
                b'C' => ProjectKind::Cooking,
                b'F' => ProjectKind::Food,
                _ => {
                    return Err(ParseCodeError {
                        kind: ParseCodeErrorKind::UnknownGroup,
                    })
                }
            },
            5 => {
                let is_online = match s[0] {
                    b'P' => false,
                    b'O' => true,
                    _ => {
                        return Err(ParseCodeError {
                            kind: ParseCodeErrorKind::UnknownOnlineFlag,
                        })
                    }
                };

                match s[1] {
                    b'G' => ProjectKind::General { is_online },
                    b'S' => ProjectKind::Stage { is_online },
                    b'C' | b'F' => {
                        return Err(ParseCodeError {
                            kind: ParseCodeErrorKind::GotExtraOnlineFlag,
                        })
                    }
                    _ => {
                        return Err(ParseCodeError {
                            kind: ParseCodeErrorKind::UnknownGroup,
                        })
                    }
                }
            }
            _ => {
                return Err(ParseCodeError {
                    kind: ParseCodeErrorKind::MismatchedLength,
                })
            }
        };

        let index_slice = match s.len(){
            4 => &s[1..4],
            5=> &s[2..5],
            _=> unreachable!()
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
        match self.kind {
            ProjectKind::General { is_online } | ProjectKind::Stage { is_online } => {
                f.write_char(if is_online { 'O' } else { 'P' })?;
            }
            _ => (),
        };

        let group = match self.kind {
            ProjectKind::General { .. } => 'G',
            ProjectKind::Stage { .. } => 'S',
            ProjectKind::Food => 'F',
            ProjectKind::Cooking => 'C',
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
            ProjectCode::parse("PG001").unwrap(),
            ProjectCode {
                kind: ProjectKind::General { is_online: false },
                index: ProjectIndex::from_u16(1).unwrap(),
            }
        );

        assert_eq!(
            ProjectCode::parse("PS000").unwrap(),
            ProjectCode {
                kind: ProjectKind::Stage { is_online: false },
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
            ProjectCode::parse("OG000").unwrap(),
            ProjectCode {
                kind: ProjectKind::General { is_online: true },
                index: ProjectIndex::from_u16(0).unwrap(),
            }
        );

        assert_eq!(
            ProjectCode::parse("OS000").unwrap(),
            ProjectCode {
                kind: ProjectKind::Stage { is_online: true },
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
            ProjectCode::parse("OG0001").unwrap_err().kind(),
            ParseCodeErrorKind::MismatchedLength
        );
        assert_eq!(
            ProjectCode::parse("OGFFF").unwrap_err().kind(),
            ParseCodeErrorKind::InvalidIndex
        );
        assert!(ProjectCode::parse("XA000").is_err());
        assert_eq!(
            ProjectCode::parse("AG001").unwrap_err().kind(),
            ParseCodeErrorKind::UnknownOnlineFlag
        );

        assert_eq!(
            ProjectCode::parse("G001").unwrap_err().kind(),
            ParseCodeErrorKind::MissingOnlineFlag
        );
        assert_eq!(
            ProjectCode::parse("S001").unwrap_err().kind(),
            ParseCodeErrorKind::MissingOnlineFlag
        );

        assert_eq!(
            ProjectCode::parse("PF001").unwrap_err().kind(),
            ParseCodeErrorKind::GotExtraOnlineFlag
        );
        assert_eq!(
            ProjectCode::parse("OC001").unwrap_err().kind(),
            ParseCodeErrorKind::GotExtraOnlineFlag
        );

        assert_eq!(
            ProjectCode::parse("A001").unwrap_err().kind(),
            ParseCodeErrorKind::UnknownGroup
        );

        assert_eq!(
            ProjectCode::parse("OA001").unwrap_err().kind(),
            ParseCodeErrorKind::UnknownGroup
        );
    }

    #[test]
    fn test_serialize_deserialize() {
        fn ser_de(code: ProjectCode) {
            assert_eq!(ProjectCode::parse(&code.to_string()).unwrap(), code);
        }

        ser_de(ProjectCode {
            kind: ProjectKind::General { is_online: false },
            index: ProjectIndex::from_u16(1).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind::Stage { is_online: false },
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
            kind: ProjectKind::General { is_online: true },
            index: ProjectIndex::from_u16(0).unwrap(),
        });

        ser_de(ProjectCode {
            kind: ProjectKind::Stage { is_online: true },
            index: ProjectIndex::from_u16(0).unwrap(),
        });
    }

    #[test]
    fn test_deserialize_serialize() {
        fn de_ser(code: &str) {
            assert_eq!(ProjectCode::parse(code).unwrap().to_string(), code);
        }

        de_ser("PG000");
        de_ser("PS001");
        de_ser("C010");
        de_ser("F011");
        de_ser("OG100");
        de_ser("OS999");
    }
}
