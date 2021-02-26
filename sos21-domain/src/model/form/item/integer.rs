use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

pub mod limit;
pub mod unit;

pub use limit::IntegerFormItemLimit;
pub use unit::IntegerFormItemUnit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegerFormItemContent {
    pub is_required: bool,
    pub max: Option<IntegerFormItemLimit>,
    pub min: Option<IntegerFormItemLimit>,
    pub placeholder: u64,
    pub unit: Option<IntegerFormItemUnit>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct IntegerFormItem(IntegerFormItemContent);

#[derive(Debug, Error, Clone)]
#[error("invalid integer form item placeholder value")]
pub struct PlaceholderError {
    _priv: (),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckAnswerErrorKind {
    NotAnswered,
    TooBig,
    TooSmall,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form answer integer item")]
pub struct CheckAnswerError {
    kind: CheckAnswerErrorKind,
}

impl CheckAnswerError {
    pub fn kind(&self) -> CheckAnswerErrorKind {
        self.kind
    }
}

impl IntegerFormItem {
    pub fn from_content(content: IntegerFormItemContent) -> Result<Self, PlaceholderError> {
        if let Some(min) = content.min {
            if content.placeholder > min.to_u64() {
                return Err(PlaceholderError { _priv: () });
            }
        }

        if let Some(max) = content.max {
            if content.placeholder > max.to_u64() {
                return Err(PlaceholderError { _priv: () });
            }
        }

        Ok(IntegerFormItem(content))
    }

    pub fn min_limit(&self) -> Option<u64> {
        self.0.min.map(IntegerFormItemLimit::to_u64)
    }

    pub fn max_limit(&self) -> Option<u64> {
        self.0.max.map(IntegerFormItemLimit::to_u64)
    }

    pub fn into_content(self) -> IntegerFormItemContent {
        self.0
    }

    pub fn check_answer(&self, answer: Option<u64>) -> Result<(), CheckAnswerError> {
        let answer = match (self.0.is_required, answer) {
            (true, None) => {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::NotAnswered,
                })
            }
            (false, None) => return Ok(()),
            (_, Some(answer)) => answer,
        };

        if let Some(max) = self.0.max {
            if max.to_u64() < answer {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::TooBig,
                });
            }
        }

        if let Some(min) = self.0.min {
            if min.to_u64() < answer {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::TooSmall,
                });
            }
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for IntegerFormItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        IntegerFormItem::from_content(IntegerFormItemContent::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}
