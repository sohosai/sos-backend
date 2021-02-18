use sos21_domain::model::form::item::radio as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RadioId(pub Uuid);

impl RadioId {
    pub fn from_entity(id: entity::RadioId) -> RadioId {
        RadioId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::RadioId {
        entity::RadioId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Radio {
    pub id: RadioId,
    pub label: String,
}

impl Radio {
    pub fn from_entity(radio: entity::Radio) -> Self {
        Radio {
            id: RadioId::from_entity(radio.id),
            label: radio.label.into_string(),
        }
    }
}
