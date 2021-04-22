use crate::model::pending_project::{PendingProject, PendingProjectId};
use crate::model::project::{Project, ProjectId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistrationFormAnswerRespondent {
    Project(ProjectId),
    PendingProject(PendingProjectId),
}

impl RegistrationFormAnswerRespondent {
    pub fn is_project(&self, project: &Project) -> bool {
        matches!(self,
            RegistrationFormAnswerRespondent::Project(id)
            if project.id == *id
        )
    }

    pub fn is_pending_project(&self, pending_project: &PendingProject) -> bool {
        matches!(self,
            RegistrationFormAnswerRespondent::PendingProject(id)
            if pending_project.id == *id
        )
    }
}
