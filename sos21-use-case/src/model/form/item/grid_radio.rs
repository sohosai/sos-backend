use sos21_domain::model::form::item::grid_radio as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridRadioRowId(pub Uuid);

impl GridRadioRowId {
    pub fn from_entity(id: entity::GridRadioRowId) -> GridRadioRowId {
        GridRadioRowId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::GridRadioRowId {
        entity::GridRadioRowId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct GridRadioRow {
    pub id: GridRadioRowId,
    pub label: String,
}

impl GridRadioRow {
    pub fn from_entity(row: entity::GridRadioRow) -> Self {
        GridRadioRow {
            id: GridRadioRowId::from_entity(row.id),
            label: row.label.into_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridRadioColumnId(pub Uuid);

impl GridRadioColumnId {
    pub fn from_entity(id: entity::GridRadioColumnId) -> GridRadioColumnId {
        GridRadioColumnId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::GridRadioColumnId {
        entity::GridRadioColumnId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct GridRadioColumn {
    pub id: GridRadioColumnId,
    pub label: String,
}

impl GridRadioColumn {
    pub fn from_entity(column: entity::GridRadioColumn) -> Self {
        GridRadioColumn {
            id: GridRadioColumnId::from_entity(column.id),
            label: column.label.into_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GridRadioRequired {
    All,
    None,
}

impl GridRadioRequired {
    pub fn from_entity(required: entity::GridRadioFormItemRequired) -> Self {
        match required {
            entity::GridRadioFormItemRequired::All => GridRadioRequired::All,
            entity::GridRadioFormItemRequired::None => GridRadioRequired::None,
        }
    }

    pub fn into_entity(self) -> entity::GridRadioFormItemRequired {
        match self {
            GridRadioRequired::All => entity::GridRadioFormItemRequired::All,
            GridRadioRequired::None => entity::GridRadioFormItemRequired::None,
        }
    }
}
