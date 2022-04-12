use crate::model::user::UserAffiliation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserCategory {
    UndergraduateStudent,
    GraduateStudent,
    AcademicStaff,
}

impl UserCategory {
    pub fn affiliation(&self) -> Option<&UserAffiliation> {
        match self {
            UserCategory::UndergraduateStudent => None,
            UserCategory::GraduateStudent => None,
            UserCategory::AcademicStaff => None,
        }
    }
}
