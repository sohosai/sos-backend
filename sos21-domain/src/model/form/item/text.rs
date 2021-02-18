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

    pub fn into_content(self) -> TextFormItemContent {
        self.0
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
