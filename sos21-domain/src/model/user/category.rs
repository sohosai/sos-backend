use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCategory {
    Undergraduate,
    GraduateStudent,
    AcademicStaff,
}

#[derive(Debug, Error, Clone)]
#[error("invalid user category")]
pub struct CategoryError {
    _priv: (),
}

impl UserCategory {
    pub fn from_string(category: impl Into<String>) -> Result<Self, CategoryError> {
        match category.into().as_str() {
            "undergraduate" => Ok(UserCategory::Undergraduate),
            "graduate_student" => Ok(UserCategory::GraduateStudent),
            "academic_staff" => Ok(UserCategory::AcademicStaff),
            _ => Err(CategoryError { _priv: () }),
        }
    }

    pub fn into_string(self) -> String {
        match self {
            UserCategory::Undergraduate => "undergraduate",
            UserCategory::GraduateStudent => "graduate_student",
            UserCategory::AcademicStaff => "academic_staff",
        }
        .to_string()
    }
}
