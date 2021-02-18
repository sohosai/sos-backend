use crate::model::string::LengthBoundedString;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormAnswerItemText(LengthBoundedString<typenum::U1, typenum::U1024, String>);

impl FormAnswerItemText {
    pub fn into_string(self) -> String {
        self.0.into_inner()
    }
}
