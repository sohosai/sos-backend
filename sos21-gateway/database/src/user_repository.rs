use anyhow::{Context, Result};
use futures::lock::Mutex;
use futures::{future, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::UserRepository;
use sos21_domain::model::{
    date_time::DateTime,
    pending_project::PendingProjectId,
    phone_number::PhoneNumber,
    project::ProjectId,
    user::{
        User, UserAffiliation, UserAssignment, UserCategory, UserContent, UserEmailAddress, UserId,
        UserKanaName, UserName, UserRole,
    },
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct UserDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl UserRepository for UserDatabase {
    async fn store_user(&self, user: User) -> Result<()> {
        let mut lock = self.0.lock().await;

        let user = from_user(user);
        if query::find_user(&mut *lock, user.id.clone())
            .await?
            .is_some()
        {
            let input = command::update_user::Input {
                id: user.id,
                first_name: user.first_name,
                kana_first_name: user.kana_first_name,
                last_name: user.last_name,
                kana_last_name: user.kana_last_name,
                phone_number: user.phone_number,
                affiliation: user.affiliation,
                role: user.role,
                category: user.category,
                assignment: user.assignment,
                assignment_owner_project_id: user.assignment_owner_project_id,
                assignment_subowner_project_id: user.assignment_subowner_project_id,
                assignment_owner_pending_project_id: user.assignment_owner_pending_project_id,
            };
            command::update_user(&mut *lock, input).await
        } else {
            command::insert_user(&mut *lock, user).await
        }
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        let mut lock = self.0.lock().await;
        query::find_user(&mut *lock, id.0)
            .await
            .and_then(|opt| opt.map(to_user).transpose())
    }

    async fn list_users(&self) -> Result<Vec<User>> {
        let mut lock = self.0.lock().await;
        query::list_users(&mut *lock)
            .and_then(|user| future::ready(to_user(user)))
            .try_collect()
            .await
    }

    async fn get_user_by_email(&self, email: &UserEmailAddress) -> Result<Option<User>> {
        let mut lock = self.0.lock().await;
        query::find_user_by_email(&mut *lock, email.as_str())
            .await
            .and_then(|opt| opt.map(to_user).transpose())
    }
}

fn from_user(user: User) -> data::user::User {
    let UserContent {
        id,
        created_at,
        name,
        kana_name,
        email,
        phone_number,
        affiliation,
        role,
        category,
        assignment,
    } = user.into_content();

    let (first_name, last_name) = name.into_string();
    let (kana_first_name, kana_last_name) = kana_name.into_string();
    let (owner_project_id, subowner_project_id, owner_pending_project_id) = match assignment {
        Some(UserAssignment::ProjectOwner(project_id)) => (Some(project_id), None, None),
        Some(UserAssignment::ProjectSubowner(project_id)) => (None, Some(project_id), None),
        Some(UserAssignment::PendingProjectOwner(pending_project_id)) => {
            (None, None, Some(pending_project_id))
        }
        None => (None, None, None),
    };

    data::user::User {
        id: id.0,
        created_at: created_at.utc(),
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        email: email.into_string(),
        phone_number: phone_number.into_string(),
        affiliation: affiliation.into_string(),
        role: match role {
            UserRole::Administrator => data::user::UserRole::Administrator,
            UserRole::CommitteeOperator => data::user::UserRole::CommitteeOperator,
            UserRole::Committee => data::user::UserRole::Committee,
            UserRole::General => data::user::UserRole::General,
        },
        category: match category {
            UserCategory::UndergraduateStudent => data::user::UserCategory::UndergraduateStudent,
            UserCategory::GraduateStudent => data::user::UserCategory::GraduateStudent,
            UserCategory::AcademicStaff => data::user::UserCategory::AcademicStaff,
        },
        assignment: assignment.map(from_user_assignment),
        assignment_owner_project_id: owner_project_id.map(|id| id.to_uuid()),
        assignment_subowner_project_id: subowner_project_id.map(|id| id.to_uuid()),
        assignment_owner_pending_project_id: owner_pending_project_id.map(|id| id.to_uuid()),
    }
}

fn from_user_assignment(assignment: UserAssignment) -> data::user::UserAssignment {
    match assignment {
        UserAssignment::ProjectOwner(_) => data::user::UserAssignment::ProjectOwner,
        UserAssignment::ProjectSubowner(_) => data::user::UserAssignment::ProjectSubowner,
        UserAssignment::PendingProjectOwner(_) => data::user::UserAssignment::PendingProjectOwner,
    }
}

pub fn to_user(user: data::user::User) -> Result<User> {
    let data::user::User {
        id,
        created_at,
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        email,
        phone_number,
        affiliation,
        role,
        category,
        assignment,
        assignment_owner_project_id,
        assignment_subowner_project_id,
        assignment_owner_pending_project_id,
    } = user;

    let assignment = if let Some(assignment) = assignment {
        Some(match assignment {
            data::user::UserAssignment::ProjectOwner => {
                let project_id = assignment_owner_project_id.context(
                    "assignment = 'project_owner' but assignment_owner_project_id is null",
                )?;
                UserAssignment::ProjectOwner(ProjectId::from_uuid(project_id))
            }
            data::user::UserAssignment::ProjectSubowner => {
                let project_id = assignment_subowner_project_id.context(
                    "assignment = 'project_subowner' but assignment_subowner_project_id is null",
                )?;
                UserAssignment::ProjectSubowner(ProjectId::from_uuid(project_id))
            }
            data::user::UserAssignment::PendingProjectOwner => {
                let pending_project_id = assignment_owner_pending_project_id.context(
                "assignment = 'pending_project_owner' but assignment_owner_pending_project_id is null",
            )?;
                UserAssignment::PendingProjectOwner(PendingProjectId::from_uuid(pending_project_id))
            }
        })
    } else {
        None
    };

    Ok(User::from_content(UserContent {
        id: UserId(id),
        created_at: DateTime::from_utc(created_at),
        name: UserName::from_string(first_name, last_name)?,
        kana_name: UserKanaName::from_string(kana_first_name, kana_last_name)?,
        email: UserEmailAddress::from_string(email)?,
        phone_number: PhoneNumber::from_string(phone_number)?,
        affiliation: UserAffiliation::from_string(affiliation)?,
        role: match role {
            data::user::UserRole::Administrator => UserRole::Administrator,
            data::user::UserRole::CommitteeOperator => UserRole::CommitteeOperator,
            data::user::UserRole::Committee => UserRole::Committee,
            data::user::UserRole::General => UserRole::General,
        },
        category: match category {
            data::user::UserCategory::UndergraduateStudent => UserCategory::UndergraduateStudent,
            data::user::UserCategory::GraduateStudent => UserCategory::GraduateStudent,
            data::user::UserCategory::AcademicStaff => UserCategory::AcademicStaff,
        },
        assignment,
    }))
}
