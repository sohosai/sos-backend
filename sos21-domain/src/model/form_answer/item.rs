use crate::model::collection::LengthBoundedVec;
use crate::model::form::item::radio::RadioId;
use crate::model::form::item::FormItemId;

use serde::{Deserialize, Serialize};

pub mod checks;
pub mod grid_rows;
pub mod text;
pub use checks::FormAnswerItemChecks;
pub use grid_rows::{FormAnswerItemGridRows, GridRadioRowAnswer};
pub use text::FormAnswerItemText;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormAnswerItems(LengthBoundedVec<typenum::U1, typenum::U64, FormAnswerItem>);

impl FormAnswerItems {
    pub fn into_items(self) -> impl Iterator<Item = FormAnswerItem> {
        self.0.into_inner().into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormAnswerItem {
    pub item_id: FormItemId,
    pub body: FormAnswerItemBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormAnswerItemBody {
    Text(Option<FormAnswerItemText>),
    Integer(Option<u64>),
    Checkbox(FormAnswerItemChecks),
    Radio(Option<RadioId>),
    GridRadio(FormAnswerItemGridRows),
}
