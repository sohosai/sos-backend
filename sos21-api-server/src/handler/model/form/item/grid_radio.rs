use serde::{Deserialize, Serialize};
use sos21_use_case::model::form::item as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GridRadioRowId(pub Uuid);

impl GridRadioRowId {
    pub fn from_use_case(id: use_case::GridRadioRowId) -> Self {
        GridRadioRowId(id.0)
    }

    pub fn into_use_case(self) -> use_case::GridRadioRowId {
        use_case::GridRadioRowId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRadioRow {
    pub id: GridRadioRowId,
    pub label: String,
}

impl GridRadioRow {
    pub fn from_use_case(button: use_case::GridRadioRow) -> Self {
        GridRadioRow {
            id: GridRadioRowId::from_use_case(button.id),
            label: button.label,
        }
    }

    pub fn into_use_case(self) -> use_case::GridRadioRow {
        use_case::GridRadioRow {
            id: self.id.into_use_case(),
            label: self.label,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GridRadioColumnId(pub Uuid);

impl GridRadioColumnId {
    pub fn from_use_case(id: use_case::GridRadioColumnId) -> Self {
        GridRadioColumnId(id.0)
    }

    pub fn into_use_case(self) -> use_case::GridRadioColumnId {
        use_case::GridRadioColumnId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRadioColumn {
    pub id: GridRadioColumnId,
    pub label: String,
}

impl GridRadioColumn {
    pub fn from_use_case(button: use_case::GridRadioColumn) -> Self {
        GridRadioColumn {
            id: GridRadioColumnId::from_use_case(button.id),
            label: button.label,
        }
    }

    pub fn into_use_case(self) -> use_case::GridRadioColumn {
        use_case::GridRadioColumn {
            id: self.id.into_use_case(),
            label: self.label,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridRadioRequired {
    All,
    None,
}

impl GridRadioRequired {
    pub fn from_use_case(required: use_case::GridRadioRequired) -> Self {
        match required {
            use_case::GridRadioRequired::All => GridRadioRequired::All,
            use_case::GridRadioRequired::None => GridRadioRequired::None,
        }
    }

    pub fn into_use_case(self) -> use_case::GridRadioRequired {
        match self {
            GridRadioRequired::All => use_case::GridRadioRequired::All,
            GridRadioRequired::None => use_case::GridRadioRequired::None,
        }
    }
}
