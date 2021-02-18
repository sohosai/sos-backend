use serde::{Deserialize, Serialize};
use sos21_use_case::model::form::item as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RadioId(pub Uuid);

impl RadioId {
    pub fn from_use_case(id: use_case::RadioId) -> Self {
        RadioId(id.0)
    }

    pub fn into_use_case(self) -> use_case::RadioId {
        use_case::RadioId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Radio {
    pub id: RadioId,
    pub label: String,
}

impl Radio {
    pub fn from_use_case(button: use_case::Radio) -> Self {
        Radio {
            id: RadioId::from_use_case(button.id),
            label: button.label,
        }
    }

    pub fn into_use_case(self) -> use_case::Radio {
        use_case::Radio {
            id: self.id.into_use_case(),
            label: self.label,
        }
    }
}
