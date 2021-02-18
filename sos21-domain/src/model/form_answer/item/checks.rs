use crate::model::bound::{Bounded, Unbounded};
use crate::model::collection::LengthLimitedVec;
use crate::model::form::item::checkbox::CheckboxId;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormAnswerItemChecks(LengthLimitedVec<Unbounded, Bounded<typenum::U32>, CheckboxId>);

impl FormAnswerItemChecks {
    pub fn checked_ids(&self) -> impl Iterator<Item = CheckboxId> + '_ {
        self.0.iter().copied()
    }
}
