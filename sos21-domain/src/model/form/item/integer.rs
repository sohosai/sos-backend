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

    pub fn into_content(self) -> IntegerFormItemContent {
        self.0
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
