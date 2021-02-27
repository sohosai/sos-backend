use serde::{Deserialize, Serialize};
use sos21_use_case::model::form::item as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckboxId(pub Uuid);

impl CheckboxId {
    pub fn from_use_case(id: use_case::CheckboxId) -> Self {
        CheckboxId(id.0)
    }

    pub fn into_use_case(self) -> use_case::CheckboxId {
        use_case::CheckboxId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkbox {
    pub id: CheckboxId,
    pub label: String,
}

impl Checkbox {
    pub fn from_use_case(checkbox: use_case::Checkbox) -> Self {
        Checkbox {
            id: CheckboxId::from_use_case(checkbox.id),
            label: checkbox.label,
        }
    }

    pub fn into_use_case(self) -> use_case::Checkbox {
        use_case::Checkbox {
            id: self.id.into_use_case(),
            label: self.label,
        }
    }
}
