use crate::model::form_answer::{FormAnswer, FormAnswerId};
use crate::model::project::{Project, ProjectId};
use crate::model::user::User;

#[derive(Debug, Clone, Copy)]
pub enum FileSharingScope {
    Project(ProjectId),
    FormAnswer(FormAnswerId),
    Committee,
    CommitteeOperator,
    Public,
}

impl FileSharingScope {
    pub fn is_public(&self) -> bool {
        matches!(self, FileSharingScope::Public)
    }

    pub fn project(&self) -> Option<ProjectId> {
        match self {
            FileSharingScope::Project(project_id) => Some(*project_id),
            _ => None,
        }
    }

    pub fn form_answer(&self) -> Option<FormAnswerId> {
        match self {
            FileSharingScope::FormAnswer(form_answer_id) => Some(*form_answer_id),
            _ => None,
        }
    }

    pub fn contains_user(&self, user: &User) -> bool {
        match self {
            FileSharingScope::Project(_) => false,
            FileSharingScope::FormAnswer(_) => false,
            FileSharingScope::CommitteeOperator => user.is_committee_operator(),
            FileSharingScope::Committee => user.is_committee(),
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_project(&self, project: &Project) -> bool {
        match self {
            FileSharingScope::Project(project_id) => *project_id == project.id,
            FileSharingScope::FormAnswer(_)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_form_answer(&self, answer: &FormAnswer) -> bool {
        match self {
            FileSharingScope::FormAnswer(answer_id) => *answer_id == answer.id,
            FileSharingScope::Project(_)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }
}
