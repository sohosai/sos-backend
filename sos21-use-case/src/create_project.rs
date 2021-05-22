use crate::error::{UseCaseError, UseCaseResult};
use crate::model::pending_project::PendingProjectId;
use crate::model::project::{Project, ProjectFromEntityInput};

use anyhow::Context;
use sos21_domain::context::{
    ConfigContext, FileSharingRepository, Login, PendingProjectRepository, ProjectRepository,
    RegistrationFormAnswerRepository, RegistrationFormRepository, UserRepository,
};
use sos21_domain::model::project;

#[derive(Debug, Clone)]
pub enum Error {
    PendingProjectNotFound,
    TooManyProjects,
    NotAnsweredRegistrationForm,
    SameOwnerSubowner,
    AlreadyProjectOwner,
    AlreadyProjectSubowner,
    AlreadyPendingProjectOwner,
    OutOfCreationPeriod,
}

impl Error {
    fn from_new_project_error(err: project::NewProjectError) -> Self {
        match err.kind() {
            project::NewProjectErrorKind::TooManyProjects => Error::TooManyProjects,
            project::NewProjectErrorKind::NotAnsweredRegistrationForm => {
                Error::NotAnsweredRegistrationForm
            }
            project::NewProjectErrorKind::SameOwnerSubowner => Error::SameOwnerSubowner,
            project::NewProjectErrorKind::AlreadyProjectOwnerSubowner => Error::AlreadyProjectOwner,
            project::NewProjectErrorKind::AlreadyProjectSubownerSubowner => {
                Error::AlreadyProjectSubowner
            }
            project::NewProjectErrorKind::AlreadyPendingProjectOwnerSubowner => {
                Error::AlreadyPendingProjectOwner
            }
            project::NewProjectErrorKind::OutOfCreationPeriod => Error::OutOfCreationPeriod,
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    pending_project_id: PendingProjectId,
) -> UseCaseResult<Project, Error>
where
    C: ProjectRepository
        + PendingProjectRepository
        + FileSharingRepository
        + RegistrationFormRepository
        + RegistrationFormAnswerRepository
        + UserRepository
        + ConfigContext
        + Send
        + Sync,
{
    let mut login_user = ctx.login_user().clone();

    let result = ctx
        .get_pending_project(pending_project_id.into_entity())
        .await
        .context("Failed to get a pending project")?;
    let (pending_project, mut owner) = match result {
        Some(result) if result.pending_project.is_visible_to(&login_user) => {
            (result.pending_project, result.owner)
        }
        _ => return Err(UseCaseError::UseCase(Error::PendingProjectNotFound)),
    };

    let pending_project_id = pending_project.id();
    let project = project::Project::new(ctx, pending_project, &login_user)
        .await
        .map_err(|err| UseCaseError::from_domain(err, Error::from_new_project_error))?;

    ctx.store_project(project.clone())
        .await
        .context("Failed to store a project")?;

    owner.assign_project_owner(&project)?;
    login_user.assign_project_subowner(&project)?;

    ctx.store_user(owner.clone())
        .await
        .context("Failed to store a user")?;
    ctx.store_user(login_user.clone())
        .await
        .context("Failed to store a user")?;

    // TODO: Split these heavy (O(n)) processes into a separate asynchronous job
    //       or reduce the number of storings
    {
        let answers = ctx
            .list_registration_form_answers_by_pending_project(pending_project_id)
            .await
            .context("Failed to list registration form answers")?;
        for mut answer in answers {
            answer.respondent.replace_to_project(&project);
            ctx.store_registration_form_answer(answer)
                .await
                .context("Failed to store a registration form answer")?;
        }

        let sharings = ctx
            .list_file_sharings_by_pending_project(pending_project_id)
            .await
            .context("Failed to list file sharings")?;
        for mut sharing in sharings {
            use_case_ensure!(sharing.scope().registration_form_answer().is_some());
            sharing.set_scope_to_project_answer(&project)?;
            ctx.store_file_sharing(sharing)
                .await
                .context("Failed to store a file sharing")?;
        }

        ctx.delete_pending_project(pending_project_id)
            .await
            .context("Failed to delete a pending project")?;
    }

    use_case_ensure!(project.is_visible_to(&login_user));
    use_case_ensure!(project.subowner_id() == login_user.id());
    Ok(Project::from_entity(ProjectFromEntityInput {
        project,
        owner_name: owner.name().clone(),
        owner_kana_name: owner.kana_name().clone(),
        subowner_name: login_user.name().clone(),
        subowner_kana_name: login_user.kana_name().clone(),
    }))
}

#[cfg(test)]
mod tests {
    use crate::model::pending_project::PendingProjectId;
    use crate::model::user::UserId;
    use crate::{create_project, get_pending_project, UseCaseError};
    use sos21_domain::test;

    #[tokio::test]
    async fn test_owner() {
        let user = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            create_project::run(&app, PendingProjectId::from_entity(pending_project.id())).await,
            Err(UseCaseError::UseCase(
                create_project::Error::SameOwnerSubowner
            ))
        ));
    }

    #[tokio::test]
    async fn test_already_project_owner() {
        let owner = test::model::new_general_user();
        let mut user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id().clone());
        user.assign_project_owner(&project).unwrap();

        let pending_project = test::model::new_general_pending_project(owner.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), owner.clone()])
            .projects(vec![project.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            create_project::run(&app, PendingProjectId::from_entity(pending_project.id())).await,
            Err(UseCaseError::UseCase(
                create_project::Error::AlreadyProjectOwner
            ))
        ));
    }

    #[tokio::test]
    async fn test_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let pending_project_id = PendingProjectId::from_entity(pending_project.id());

        assert!(matches!(
            create_project::run(&app, pending_project_id).await,
            Ok(got)
            if got.owner_id == UserId::from_entity(other.id().clone())
            && got.subowner_id == UserId::from_entity(user.id().clone())
        ));

        assert!(matches!(
            get_pending_project::run(&app, pending_project_id).await,
            Err(UseCaseError::UseCase(get_pending_project::Error::NotFound))
        ));
    }

    #[tokio::test]
    async fn test_with_period_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(other.id().clone());
        let period = test::model::new_project_creation_period_from_now();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .project_creation_period(period)
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            create_project::run(
                &app,
                PendingProjectId::from_entity(pending_project.id()),
            )
            .await,
            Ok(got)
            if got.owner_id == UserId::from_entity(other.id().clone())
            && got.subowner_id == UserId::from_entity(user.id().clone())
        ));
    }

    #[tokio::test]
    async fn test_after_period_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(other.id().clone());
        let period = test::model::new_project_creation_period_to_now();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .project_creation_period(period)
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            create_project::run(&app, PendingProjectId::from_entity(pending_project.id())).await,
            Err(UseCaseError::UseCase(
                create_project::Error::OutOfCreationPeriod
            ))
        ));
    }

    #[tokio::test]
    async fn test_before_period_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(other.id().clone());
        let period = test::model::new_project_creation_period_with_hours_from_now(1);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .project_creation_period(period)
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            create_project::run(&app, PendingProjectId::from_entity(pending_project.id())).await,
            Err(UseCaseError::UseCase(
                create_project::Error::OutOfCreationPeriod
            ))
        ));
    }

    #[tokio::test]
    async fn test_other_with_registration_form_not_answered() {
        let owner = test::model::new_general_user();
        let subowner = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(owner.id().clone());

        let operator = test::model::new_general_user();
        let registration_form1 = test::model::new_registration_form(operator.id().clone());
        let registration_form2 = test::model::new_registration_form(operator.id().clone());

        let answer1 = test::model::new_registration_form_answer_with_pending_project(
            owner.id().clone(),
            pending_project.id(),
            &registration_form1,
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form1.clone(), registration_form2.clone()])
            .registration_form_answers(vec![answer1.clone()])
            .build()
            .login_as(subowner.clone())
            .await;

        assert!(matches!(
            create_project::run(&app, PendingProjectId::from_entity(pending_project.id())).await,
            Err(UseCaseError::UseCase(
                create_project::Error::NotAnsweredRegistrationForm { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn test_other_with_registration_form_answered() {
        use sos21_domain::model::{project, project_query};

        let owner = test::model::new_general_user();
        let subowner = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(owner.id().clone());

        let operator = test::model::new_general_user();
        let registration_form1 = test::model::new_registration_form(operator.id().clone());
        let registration_form2 = test::model::new_registration_form(operator.id().clone());
        let registration_form3 = test::model::new_registration_form_with_query(
            operator.id().clone(),
            project_query::ProjectQuery::from_conjunctions(vec![
                project_query::ProjectQueryConjunction {
                    category: Some(project::ProjectCategory::Stage),
                    attributes: project::ProjectAttributes::from_attributes(vec![]).unwrap(),
                },
            ])
            .unwrap(),
        );

        let answer1 = test::model::new_registration_form_answer_with_pending_project(
            owner.id().clone(),
            pending_project.id(),
            &registration_form1,
        );
        let answer2 = test::model::new_registration_form_answer_with_pending_project(
            owner.id().clone(),
            pending_project.id(),
            &registration_form2,
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![
                registration_form1.clone(),
                registration_form2.clone(),
                registration_form3.clone(),
            ])
            .registration_form_answers(vec![answer1.clone(), answer2.clone()])
            .build()
            .login_as(subowner.clone())
            .await;

        let pending_project_id = PendingProjectId::from_entity(pending_project.id());

        assert!(matches!(
            create_project::run(&app, pending_project_id).await,
            Ok(got)
            if got.owner_id == UserId::from_entity(owner.id().clone())
            && got.subowner_id == UserId::from_entity(subowner.id().clone())
        ));

        assert!(matches!(
            get_pending_project::run(&app, pending_project_id).await,
            Err(UseCaseError::UseCase(get_pending_project::Error::NotFound))
        ));
    }

    #[tokio::test]
    async fn test_other_with_registration_form_subowner_visibility() {
        use crate::model::{
            file::FileId, file_sharing::FileSharingId, registration_form::RegistrationFormId,
            registration_form_answer::RegistrationFormAnswerId,
        };
        use crate::{
            get_project_registration_form_answer, get_project_registration_form_answer_shared_file,
        };

        use sos21_domain::model::{file_sharing, registration_form_answer};

        let owner = test::model::new_general_user();
        let subowner = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(owner.id().clone());

        let operator = test::model::new_general_user();
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let answer = test::model::new_registration_form_answer_with_pending_project(
            owner.id().clone(),
            pending_project.id(),
            &registration_form,
        );
        let (file, object) = test::model::new_file(owner.id().clone());
        let sharing = file_sharing::FileSharing::new(
            file.id,
            file_sharing::FileSharingScope::RegistrationFormAnswer(
                registration_form_answer::RegistrationFormAnswerRespondent::PendingProject(
                    pending_project.id(),
                ),
                registration_form.id,
            ),
        );

        let subowner_app = test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer.clone()])
            .sharings(vec![sharing.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(subowner.clone())
            .await;

        let pending_project_id = PendingProjectId::from_entity(pending_project.id());
        let project = create_project::run(&subowner_app, pending_project_id)
            .await
            .unwrap();
        assert_eq!(project.owner_id, UserId::from_entity(owner.id().clone()));
        assert_eq!(
            project.subowner_id,
            UserId::from_entity(subowner.id().clone())
        );

        let registration_form_id = RegistrationFormId::from_entity(registration_form.id);
        assert!(matches!(
            get_project_registration_form_answer::run(&subowner_app, project.id, registration_form_id).await,
            Ok(got)
            if got.id == RegistrationFormAnswerId::from_entity(answer.id)
        ));

        let input = get_project_registration_form_answer_shared_file::Input {
            project_id: project.id,
            registration_form_id,
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_registration_form_answer_shared_file::run(&subowner_app, input).await,
            Ok(got)
            if got.id == FileId::from_entity(file.id)
        ));
    }
}
