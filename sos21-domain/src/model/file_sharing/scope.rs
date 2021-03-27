use crate::model::form::{Form, FormId};
use crate::model::form_answer::FormAnswer;
use crate::model::project::{Project, ProjectId};
use crate::model::user::User;

#[derive(Debug, Clone, Copy)]
pub enum FileSharingScope {
    Project(ProjectId),
    FormAnswer(ProjectId, FormId),
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

    pub fn form_answer(&self) -> Option<(ProjectId, FormId)> {
        match self {
            FileSharingScope::FormAnswer(project_id, form_id) => Some((*project_id, *form_id)),
            _ => None,
        }
    }

    pub fn contains_user(&self, user: &User) -> bool {
        match self {
            FileSharingScope::Project(_) => false,
            FileSharingScope::FormAnswer(_, _) => false,
            FileSharingScope::CommitteeOperator => user.is_committee_operator(),
            FileSharingScope::Committee => user.is_committee(),
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_project(&self, project: &Project) -> bool {
        match self {
            FileSharingScope::Project(project_id) => *project_id == project.id,
            FileSharingScope::FormAnswer(_, _)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_form_answer(&self, answer: &FormAnswer) -> bool {
        match self {
            FileSharingScope::FormAnswer(project_id, form_id) => {
                *project_id == answer.project_id && *form_id == answer.form_id
            }
            FileSharingScope::Project(_)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_project_form_answer(&self, project: &Project, form: &Form) -> bool {
        match self {
            FileSharingScope::FormAnswer(project_id, form_id) => {
                *project_id == project.id && *form_id == form.id
            }
            FileSharingScope::Project(_)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }
}
