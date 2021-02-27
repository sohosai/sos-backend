use sos21_domain::model::form::item::checkbox as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CheckboxId(pub Uuid);

impl CheckboxId {
    pub fn from_entity(id: entity::CheckboxId) -> CheckboxId {
        CheckboxId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::CheckboxId {
        entity::CheckboxId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Checkbox {
    pub id: CheckboxId,
    pub label: String,
}

impl Checkbox {
    pub fn from_entity(checkbox: entity::Checkbox) -> Self {
        Checkbox {
            id: CheckboxId::from_entity(checkbox.id),
            label: checkbox.label.into_string(),
        }
    }
}
