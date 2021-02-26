use crate::model::form_answer::item::FormAnswerItemText;

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

pub mod length;
pub mod placeholder;

pub use length::TextFormItemLength;
pub use placeholder::TextFormItemPlaceholder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormItemContent {
    pub accept_multiple_lines: bool,
    pub is_required: bool,
    pub max_length: Option<TextFormItemLength>,
    pub min_length: Option<TextFormItemLength>,
    pub placeholder: TextFormItemPlaceholder,
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct TextFormItem(TextFormItemContent);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromContentErrorKind {
    TooLongPlaceholder,
    TooShortPlaceholder,
    InconsistentLengthLimits,
}

#[derive(Debug, Error, Clone)]
#[error("invalid text form item")]
pub struct FromContentError {
    kind: FromContentErrorKind,
}

impl FromContentError {
    pub fn kind(&self) -> FromContentErrorKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerErrorKind {
    NotAnswered,
    NotAllowedMultipleLine,
    TooLong,
    TooShort,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer text item")]
pub struct CheckAnswerError {
    kind: CheckAnswerErrorKind,
}

impl CheckAnswerError {
    pub fn kind(&self) -> CheckAnswerErrorKind {
        self.kind
    }
}

impl TextFormItem {
    pub fn from_content(content: TextFormItemContent) -> Result<Self, FromContentError> {
        match (&content.min_length, &content.max_length) {
            (Some(min_length), Some(max_length)) if min_length > max_length => {
                return Err(FromContentError {
                    kind: FromContentErrorKind::InconsistentLengthLimits,
                });
            }
            _ => {}
        }

        if let Some(min_length) = &content.min_length {
            if min_length.to_u64() > content.placeholder.len() as u64 {
                return Err(FromContentError {
                    kind: FromContentErrorKind::TooShortPlaceholder,
                });
            }
        }

        if let Some(max_length) = &content.max_length {
            if max_length.to_u64() < content.placeholder.len() as u64 {
                return Err(FromContentError {
                    kind: FromContentErrorKind::TooLongPlaceholder,
                });
            }
        }

        Ok(TextFormItem(content))
    }

    pub fn min_length(&self) -> Option<u64> {
        self.0.min_length.map(TextFormItemLength::to_u64)
    }

    pub fn max_length(&self) -> Option<u64> {
        self.0.max_length.map(TextFormItemLength::to_u64)
    }

    pub fn into_content(self) -> TextFormItemContent {
        self.0
    }

    pub fn check_answer(
        &self,
        answer: Option<&FormAnswerItemText>,
    ) -> Result<(), CheckAnswerError> {
        let answer = match (self.0.is_required, answer) {
            (true, None) => {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::NotAnswered,
                })
            }
            (false, None) => return Ok(()),
            (_, Some(answer)) => answer,
        };

        if !self.0.accept_multiple_lines && answer.contains_line_break() {
            return Err(CheckAnswerError {
                kind: CheckAnswerErrorKind::NotAllowedMultipleLine,
            });
        }

        if let Some(max_length) = self.0.max_length {
            if max_length.to_u64() < answer.len() as u64 {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::TooLong,
                });
            }
        }

        if let Some(min_length) = self.0.min_length {
            if min_length.to_u64() > answer.len() as u64 {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::TooShort,
                });
            }
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for TextFormItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        TextFormItem::from_content(TextFormItemContent::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CheckAnswerErrorKind, FromContentErrorKind, TextFormItem, TextFormItemContent,
        TextFormItemLength, TextFormItemPlaceholder,
    };

    #[test]
    fn test_pass() {
        TextFormItem::from_content(TextFormItemContent {
            accept_multiple_lines: true,
            is_required: true,
            max_length: None,
            min_length: None,
            placeholder: TextFormItemPlaceholder::from_string("").unwrap(),
        })
        .unwrap();

        TextFormItem::from_content(TextFormItemContent {
            accept_multiple_lines: true,
            is_required: true,
            max_length: None,
            min_length: None,
            placeholder: TextFormItemPlaceholder::from_string("あああああ").unwrap(),
        })
        .unwrap();

        TextFormItem::from_content(TextFormItemContent {
            accept_multiple_lines: true,
            is_required: true,
            max_length: Some(TextFormItemLength::from_u64(3).unwrap()),
            min_length: None,
            placeholder: TextFormItemPlaceholder::from_string("").unwrap(),
        })
        .unwrap();

        TextFormItem::from_content(TextFormItemContent {
            accept_multiple_lines: true,
            is_required: true,
            max_length: Some(TextFormItemLength::from_u64(5).unwrap()),
            min_length: Some(TextFormItemLength::from_u64(1).unwrap()),
            placeholder: TextFormItemPlaceholder::from_string("あああああ").unwrap(),
        })
        .unwrap();
    }

    #[test]
    fn test_placeholder() {
        assert_eq!(
            TextFormItem::from_content(TextFormItemContent {
                accept_multiple_lines: true,
                is_required: true,
                max_length: Some(TextFormItemLength::from_u64(2).unwrap()),
                min_length: Some(TextFormItemLength::from_u64(1).unwrap()),
                placeholder: TextFormItemPlaceholder::from_string("あああ").unwrap(),
            })
            .unwrap_err()
            .kind(),
            FromContentErrorKind::TooLongPlaceholder,
        );

        assert_eq!(
            TextFormItem::from_content(TextFormItemContent {
                accept_multiple_lines: true,
                is_required: true,
                max_length: Some(TextFormItemLength::from_u64(2).unwrap()),
                min_length: Some(TextFormItemLength::from_u64(1).unwrap()),
                placeholder: TextFormItemPlaceholder::from_string("").unwrap(),
            })
            .unwrap_err()
            .kind(),
            FromContentErrorKind::TooShortPlaceholder,
        );
    }

    #[test]
    fn test_inconsistent() {
        assert_eq!(
            TextFormItem::from_content(TextFormItemContent {
                accept_multiple_lines: true,
                is_required: true,
                max_length: Some(TextFormItemLength::from_u64(1).unwrap()),
                min_length: Some(TextFormItemLength::from_u64(2).unwrap()),
                placeholder: TextFormItemPlaceholder::from_string("あ").unwrap(),
            })
            .unwrap_err()
            .kind(),
            FromContentErrorKind::InconsistentLengthLimits,
        );
    }

    #[test]
    fn test_answer_pass() {
        use crate::model::form_answer::item::FormAnswerItemText;

        TextFormItem::from_content(TextFormItemContent {
            accept_multiple_lines: true,
            is_required: true,
            max_length: None,
            min_length: None,
            placeholder: TextFormItemPlaceholder::from_string("").unwrap(),
        })
        .unwrap()
        .check_answer(Some(
            &FormAnswerItemText::from_string("あああ\nあいうえお").unwrap(),
        ))
        .unwrap();

        TextFormItem::from_content(TextFormItemContent {
            accept_multiple_lines: true,
            is_required: false,
            max_length: None,
            min_length: None,
            placeholder: TextFormItemPlaceholder::from_string("").unwrap(),
        })
        .unwrap()
        .check_answer(None)
        .unwrap();

        TextFormItem::from_content(TextFormItemContent {
            accept_multiple_lines: true,
            is_required: false,
            max_length: Some(TextFormItemLength::from_u64(2).unwrap()),
            min_length: Some(TextFormItemLength::from_u64(1).unwrap()),
            placeholder: TextFormItemPlaceholder::from_string("あ").unwrap(),
        })
        .unwrap()
        .check_answer(Some(&FormAnswerItemText::from_string("い").unwrap()))
        .unwrap();
    }

    #[test]
    fn test_answer_not_answered() {
        assert_eq!(
            TextFormItem::from_content(TextFormItemContent {
                accept_multiple_lines: true,
                is_required: true,
                max_length: None,
                min_length: None,
                placeholder: TextFormItemPlaceholder::from_string("").unwrap(),
            })
            .unwrap()
            .check_answer(None)
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAnswered,
        );
    }

    #[test]
    fn test_answer_multiple_line() {
        use crate::model::form_answer::item::FormAnswerItemText;

        assert_eq!(
            TextFormItem::from_content(TextFormItemContent {
                accept_multiple_lines: false,
                is_required: true,
                max_length: None,
                min_length: None,
                placeholder: TextFormItemPlaceholder::from_string("").unwrap(),
            })
            .unwrap()
            .check_answer(Some(
                &FormAnswerItemText::from_string("あああ\nあいうえお").unwrap()
            ))
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAllowedMultipleLine,
        );
    }

    #[test]
    fn test_answer_length() {
        use crate::model::form_answer::item::FormAnswerItemText;

        assert_eq!(
            TextFormItem::from_content(TextFormItemContent {
                accept_multiple_lines: true,
                is_required: true,
                max_length: None,
                min_length: Some(TextFormItemLength::from_u64(2).unwrap()),
                placeholder: TextFormItemPlaceholder::from_string("あい").unwrap(),
            })
            .unwrap()
            .check_answer(Some(&FormAnswerItemText::from_string("あ").unwrap()))
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::TooShort,
        );

        assert_eq!(
            TextFormItem::from_content(TextFormItemContent {
                accept_multiple_lines: true,
                is_required: true,
                max_length: Some(TextFormItemLength::from_u64(4).unwrap()),
                min_length: None,
                placeholder: TextFormItemPlaceholder::from_string("あ").unwrap(),
            })
            .unwrap()
            .check_answer(Some(
                &FormAnswerItemText::from_string("あいうえお").unwrap()
            ))
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::TooLong,
        );
    }
}
