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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromContentErrorKind {
    TooBigPlaceholder,
    TooSmallPlaceholder,
    InconsistentLimits,
}

#[derive(Debug, Error, Clone)]
#[error("invalid integer form item")]
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
    pub fn from_content(content: IntegerFormItemContent) -> Result<Self, FromContentError> {
        match (content.min, content.max) {
            (Some(min), Some(max)) if min > max => {
                return Err(FromContentError {
                    kind: FromContentErrorKind::InconsistentLimits,
                });
            }
            _ => {}
        }

        if let Some(min) = content.min {
            if min.to_u64() > content.placeholder {
                return Err(FromContentError {
                    kind: FromContentErrorKind::TooSmallPlaceholder,
                });
            }
        }

        if let Some(max) = content.max {
            if max.to_u64() < content.placeholder {
                return Err(FromContentError {
                    kind: FromContentErrorKind::TooBigPlaceholder,
                });
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

        if let Some(min) = self.0.min {
            if min.to_u64() > answer {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::TooSmall,
                });
            }
        }

        if let Some(max) = self.0.max {
            if max.to_u64() < answer {
                return Err(CheckAnswerError {
                    kind: CheckAnswerErrorKind::TooBig,
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

#[cfg(test)]
mod tests {
    use super::{
        CheckAnswerErrorKind, FromContentErrorKind, IntegerFormItem, IntegerFormItemContent,
        IntegerFormItemLimit,
    };

    #[test]
    fn test_pass() {
        IntegerFormItem::from_content(IntegerFormItemContent {
            is_required: true,
            max: None,
            min: None,
            placeholder: 0,
            unit: None,
        })
        .unwrap();

        IntegerFormItem::from_content(IntegerFormItemContent {
            is_required: true,
            max: Some(IntegerFormItemLimit::from_u64(3).unwrap()),
            min: Some(IntegerFormItemLimit::from_u64(1).unwrap()),
            placeholder: 1,
            unit: None,
        })
        .unwrap();

        IntegerFormItem::from_content(IntegerFormItemContent {
            is_required: true,
            max: Some(IntegerFormItemLimit::from_u64(3).unwrap()),
            min: Some(IntegerFormItemLimit::from_u64(2).unwrap()),
            placeholder: 3,
            unit: None,
        })
        .unwrap();

        IntegerFormItem::from_content(IntegerFormItemContent {
            is_required: true,
            max: Some(IntegerFormItemLimit::from_u64(4).unwrap()),
            min: Some(IntegerFormItemLimit::from_u64(2).unwrap()),
            placeholder: 3,
            unit: None,
        })
        .unwrap();
    }

    #[test]
    fn test_placeholder() {
        assert_eq!(
            IntegerFormItem::from_content(IntegerFormItemContent {
                is_required: true,
                max: Some(IntegerFormItemLimit::from_u64(4).unwrap()),
                min: Some(IntegerFormItemLimit::from_u64(2).unwrap()),
                placeholder: 1,
                unit: None,
            })
            .unwrap_err()
            .kind(),
            FromContentErrorKind::TooSmallPlaceholder,
        );

        assert_eq!(
            IntegerFormItem::from_content(IntegerFormItemContent {
                is_required: true,
                max: Some(IntegerFormItemLimit::from_u64(4).unwrap()),
                min: Some(IntegerFormItemLimit::from_u64(2).unwrap()),
                placeholder: 5,
                unit: None,
            })
            .unwrap_err()
            .kind(),
            FromContentErrorKind::TooBigPlaceholder,
        );
    }

    #[test]
    fn test_inconsistent() {
        assert_eq!(
            IntegerFormItem::from_content(IntegerFormItemContent {
                is_required: true,
                max: Some(IntegerFormItemLimit::from_u64(2).unwrap()),
                min: Some(IntegerFormItemLimit::from_u64(4).unwrap()),
                placeholder: 3,
                unit: None,
            })
            .unwrap_err()
            .kind(),
            FromContentErrorKind::InconsistentLimits,
        );
    }

    #[test]
    fn test_answer_pass() {
        IntegerFormItem::from_content(IntegerFormItemContent {
            is_required: true,
            max: None,
            min: None,
            placeholder: 0,
            unit: None,
        })
        .unwrap()
        .check_answer(Some(1000))
        .unwrap();

        IntegerFormItem::from_content(IntegerFormItemContent {
            is_required: false,
            max: None,
            min: None,
            placeholder: 0,
            unit: None,
        })
        .unwrap()
        .check_answer(None)
        .unwrap();

        IntegerFormItem::from_content(IntegerFormItemContent {
            is_required: false,
            max: Some(IntegerFormItemLimit::from_u64(4).unwrap()),
            min: Some(IntegerFormItemLimit::from_u64(2).unwrap()),
            placeholder: 3,
            unit: None,
        })
        .unwrap()
        .check_answer(Some(3))
        .unwrap();
    }

    #[test]
    fn test_answer_not_answered() {
        assert_eq!(
            IntegerFormItem::from_content(IntegerFormItemContent {
                is_required: true,
                max: None,
                min: None,
                placeholder: 0,
                unit: None,
            })
            .unwrap()
            .check_answer(None)
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::NotAnswered,
        );
    }

    #[test]
    fn test_answer_quantitiy() {
        assert_eq!(
            IntegerFormItem::from_content(IntegerFormItemContent {
                is_required: true,
                max: None,
                min: Some(IntegerFormItemLimit::from_u64(2).unwrap()),
                placeholder: 2,
                unit: None,
            })
            .unwrap()
            .check_answer(Some(0))
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::TooSmall,
        );

        assert_eq!(
            IntegerFormItem::from_content(IntegerFormItemContent {
                is_required: true,
                max: Some(IntegerFormItemLimit::from_u64(4).unwrap()),
                min: None,
                placeholder: 0,
                unit: None,
            })
            .unwrap()
            .check_answer(Some(5))
            .unwrap_err()
            .kind(),
            CheckAnswerErrorKind::TooBig,
        );
    }
}
