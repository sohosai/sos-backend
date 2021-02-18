use crate::model::collection::LengthBoundedVec;
use crate::model::form::item::grid_radio::{GridRadioColumnId, GridRadioRowId};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormAnswerItemGridRows(LengthBoundedVec<typenum::U1, typenum::U32, GridRadioRowAnswer>);

impl FormAnswerItemGridRows {
    pub fn into_row_answers(self) -> impl Iterator<Item = GridRadioRowAnswer> {
        self.0.into_inner().into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridRadioRowAnswer {
    pub row_id: GridRadioRowId,
    pub value: Option<GridRadioColumnId>,
}
