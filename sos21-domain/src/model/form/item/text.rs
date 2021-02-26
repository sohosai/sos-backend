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

#[derive(Debug, Error, Clone)]
#[error("invalid text form item placeholder length")]
pub struct PlaceholderLengthError {
    _priv: (),
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
    pub fn from_content(content: TextFormItemContent) -> Result<Self, PlaceholderLengthError> {
        if let Some(min_length) = &content.min_length {
            if content.placeholder.len() > min_length.to_u64() as usize {
                return Err(PlaceholderLengthError { _priv: () });
            }
        }

        if let Some(max_length) = &content.max_length {
            if content.placeholder.len() > max_length.to_u64() as usize {
                return Err(PlaceholderLengthError { _priv: () });
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
