use crate::model::user::UserAffiliation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserCategory {
    UndergraduateStudent(UserAffiliation),
    GraduateStudent,
    AcademicStaff,
}

impl UserCategory {
    pub fn affiliation(&self) -> Option<&UserAffiliation> {
        match self {
            UserCategory::UndergraduateStudent(affiliation) => Some(&affiliation),
            UserCategory::GraduateStudent => None,
            UserCategory::AcademicStaff => None,
        }
    }
}
