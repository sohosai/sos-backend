use crate::handler::model::date_time::DateTime;
use crate::handler::model::form_answer::FormAnswerItem;
use crate::handler::model::pending_project::PendingProjectId;
use crate::handler::model::project::ProjectId;
use crate::handler::model::registration_form::RegistrationFormId;
use crate::handler::model::user::UserId;

use serde::{Deserialize, Serialize};
use sos21_use_case::model::registration_form_answer as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegistrationFormAnswerId(pub Uuid);

impl RegistrationFormAnswerId {
    pub fn from_use_case(id: use_case::RegistrationFormAnswerId) -> Self {
        RegistrationFormAnswerId(id.0)
    }

    pub fn into_use_case(self) -> use_case::RegistrationFormAnswerId {
        use_case::RegistrationFormAnswerId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationFormAnswerRespondent {
    ProjectId(ProjectId),
    PendingProjectId(PendingProjectId),
}

impl RegistrationFormAnswerRespondent {
    pub fn from_use_case(respondent: use_case::RegistrationFormAnswerRespondent) -> Self {
        match respondent {
            use_case::RegistrationFormAnswerRespondent::PendingProject(pending_project_id) => {
                RegistrationFormAnswerRespondent::PendingProjectId(PendingProjectId::from_use_case(
                    pending_project_id,
                ))
            }
            use_case::RegistrationFormAnswerRespondent::Project(project_id) => {
                RegistrationFormAnswerRespondent::ProjectId(ProjectId::from_use_case(project_id))
            }
        }
    }

    pub fn into_use_case(self) -> use_case::RegistrationFormAnswerRespondent {
        match self {
            RegistrationFormAnswerRespondent::PendingProjectId(id) => {
                use_case::RegistrationFormAnswerRespondent::PendingProject(id.into_use_case())
            }
            RegistrationFormAnswerRespondent::ProjectId(id) => {
                use_case::RegistrationFormAnswerRespondent::Project(id.into_use_case())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationFormAnswer {
    pub id: RegistrationFormAnswerId,
    #[serde(flatten)]
    pub respondent: RegistrationFormAnswerRespondent,
    pub registration_form_id: RegistrationFormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub items: Vec<FormAnswerItem>,
}

impl RegistrationFormAnswer {
    pub fn from_use_case(answer: use_case::RegistrationFormAnswer) -> Self {
        RegistrationFormAnswer {
            id: RegistrationFormAnswerId::from_use_case(answer.id),
            respondent: RegistrationFormAnswerRespondent::from_use_case(answer.respondent),
            registration_form_id: RegistrationFormId::from_use_case(answer.registration_form_id),
            created_at: DateTime::from_use_case(answer.created_at),
            author_id: UserId::from_use_case(answer.author_id),
            items: answer
                .items
                .into_iter()
                .map(FormAnswerItem::from_use_case)
                .collect(),
        }
    }
}
