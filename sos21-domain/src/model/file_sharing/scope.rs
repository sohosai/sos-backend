use crate::model::form::{Form, FormId};
use crate::model::form_answer::FormAnswer;
use crate::model::pending_project::PendingProject;
use crate::model::project::{Project, ProjectId};
use crate::model::project_query::ProjectQuery;
use crate::model::registration_form::{RegistrationForm, RegistrationFormId};
use crate::model::registration_form_answer::{
    RegistrationFormAnswer, RegistrationFormAnswerRespondent,
};
use crate::model::user::User;

#[derive(Debug, Clone)]
pub enum FileSharingScope {
    Project(ProjectId),
    ProjectQuery(ProjectQuery),
    FormAnswer(ProjectId, FormId),
    RegistrationFormAnswer(RegistrationFormAnswerRespondent, RegistrationFormId),
    Committee,
    CommitteeOperator,
    Public,
}

impl FileSharingScope {
    pub fn form_answer_scope(answer: &FormAnswer) -> Self {
        FileSharingScope::FormAnswer(answer.project_id(), answer.form_id())
    }

    pub fn is_public(&self) -> bool {
        matches!(self, FileSharingScope::Public)
    }

    pub fn project(&self) -> Option<ProjectId> {
        match self {
            FileSharingScope::Project(project_id) => Some(*project_id),
            _ => None,
        }
    }

    pub fn project_query(&self) -> Option<&ProjectQuery> {
        match self {
            FileSharingScope::ProjectQuery(query) => Some(query),
            _ => None,
        }
    }

    pub fn form_answer(&self) -> Option<(ProjectId, FormId)> {
        match self {
            FileSharingScope::FormAnswer(project_id, form_id) => Some((*project_id, *form_id)),
            _ => None,
        }
    }

    pub fn registration_form_answer(
        &self,
    ) -> Option<(RegistrationFormAnswerRespondent, RegistrationFormId)> {
        match self {
            FileSharingScope::RegistrationFormAnswer(respondent, registration_form_id) => {
                Some((*respondent, *registration_form_id))
            }
            _ => None,
        }
    }

    pub fn contains_user(&self, user: &User) -> bool {
        match self {
            FileSharingScope::CommitteeOperator => user.is_committee_operator(),
            FileSharingScope::Committee => user.is_committee(),
            FileSharingScope::Project(_)
            | FileSharingScope::ProjectQuery(_)
            | FileSharingScope::FormAnswer(_, _)
            | FileSharingScope::RegistrationFormAnswer(_, _) => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_project(&self, project: &Project) -> bool {
        match self {
            FileSharingScope::Project(project_id) => *project_id == project.id(),
            FileSharingScope::ProjectQuery(query) => query.check_project(project),
            FileSharingScope::FormAnswer(_, _)
            | FileSharingScope::RegistrationFormAnswer(_, _)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_form_answer(&self, answer: &FormAnswer) -> bool {
        match self {
            FileSharingScope::FormAnswer(project_id, form_id) => {
                *project_id == answer.project_id() && *form_id == answer.form_id()
            }
            FileSharingScope::Project(_)
            | FileSharingScope::ProjectQuery(_)
            | FileSharingScope::RegistrationFormAnswer(_, _)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_project_form_answer(&self, project: &Project, form: &Form) -> bool {
        match self {
            FileSharingScope::FormAnswer(project_id, form_id) => {
                *project_id == project.id() && *form_id == form.id()
            }
            FileSharingScope::Project(_)
            | FileSharingScope::ProjectQuery(_)
            | FileSharingScope::RegistrationFormAnswer(_, _)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_registration_form_answer(&self, answer: &RegistrationFormAnswer) -> bool {
        match self {
            FileSharingScope::RegistrationFormAnswer(respondent, registration_form_id) => {
                *respondent == answer.respondent()
                    && *registration_form_id == answer.registration_form_id()
            }
            FileSharingScope::Project(_)
            | FileSharingScope::ProjectQuery(_)
            | FileSharingScope::FormAnswer(_, _)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_pending_project_registration_form_answer(
        &self,
        pending_project: &PendingProject,
        registration_form: &RegistrationForm,
    ) -> bool {
        match self {
            FileSharingScope::RegistrationFormAnswer(respondent, registration_form_id) => {
                respondent.is_pending_project(pending_project)
                    && *registration_form_id == registration_form.id
            }
            FileSharingScope::Project(_)
            | FileSharingScope::ProjectQuery(_)
            | FileSharingScope::FormAnswer(_, _)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }

    pub fn contains_project_registration_form_answer(
        &self,
        project: &Project,
        registration_form: &RegistrationForm,
    ) -> bool {
        match self {
            FileSharingScope::RegistrationFormAnswer(respondent, registration_form_id) => {
                respondent.is_project(project) && *registration_form_id == registration_form.id
            }
            FileSharingScope::Project(_)
            | FileSharingScope::ProjectQuery(_)
            | FileSharingScope::FormAnswer(_, _)
            | FileSharingScope::Committee
            | FileSharingScope::CommitteeOperator => false,
            FileSharingScope::Public => true,
        }
    }
}
