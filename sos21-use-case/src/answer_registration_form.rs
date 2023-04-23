use crate::error::{UseCaseError, UseCaseResult};
use crate::interface;
use crate::model::form::item::FormItemId;
use crate::model::pending_project::PendingProjectId;
use crate::model::registration_form::RegistrationFormId;
use crate::model::registration_form_answer::RegistrationFormAnswer;

use anyhow::Context;
use sos21_domain::context::{
    ConfigContext, FileRepository, FileSharingRepository, Login, PendingProjectRepository,
    RegistrationFormAnswerRepository, RegistrationFormRepository,
};
use sos21_domain::model::{permissions, registration_form, user};

#[derive(Debug, Clone)]
pub struct Input {
    pub pending_project_id: PendingProjectId,
    pub registration_form_id: RegistrationFormId,
    pub items: Vec<interface::form_answer::InputFormAnswerItem>,
}

#[derive(Debug, Clone)]
pub enum Error {
    PendingProjectNotFound,
    RegistrationFormNotFound,
    AlreadyAnswered,
    OutOfProjectCreationPeriod,
    InvalidItems(interface::form_answer::FormAnswerItemsError),
    InvalidAnswer(interface::form::CheckAnswerError),
    InsufficientPermissions,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }

    fn from_items_error(err: interface::form_answer::FormAnswerItemsError) -> Self {
        Error::InvalidItems(err)
    }

    fn from_answer_error(err: registration_form::AnswerError) -> Self {
        match err.kind() {
            registration_form::AnswerErrorKind::NotTargeted => Error::RegistrationFormNotFound,
            registration_form::AnswerErrorKind::AlreadyAnswered => Error::AlreadyAnswered,
            registration_form::AnswerErrorKind::MismatchedItemsLength => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::MismatchedItemsLength)
            }
            registration_form::AnswerErrorKind::MismatchedItemId { expected, got } => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::MismatchedItemId {
                    expected: FormItemId::from_entity(expected),
                    got: FormItemId::from_entity(got),
                })
            }
            registration_form::AnswerErrorKind::InvalidItem { id, kind } => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::InvalidAnswerItem {
                    item_id: FormItemId::from_entity(id),
                    item_error: interface::form::to_check_answer_item_error(kind),
                })
            }
            registration_form::AnswerErrorKind::OutOfProjectCreationPeriod => {
                Error::OutOfProjectCreationPeriod
            }
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<RegistrationFormAnswer, Error>
where
    C: PendingProjectRepository
        + RegistrationFormRepository
        + RegistrationFormAnswerRepository
        + FileRepository
        + FileSharingRepository
        + ConfigContext
        + Send
        + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(
            permissions::Permissions::ANSWER_REGISTRATION_FORMS
                | permissions::Permissions::SHARE_FILES,
        )
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let result = ctx
        .get_pending_project(input.pending_project_id.into_entity())
        .await
        .context("Failed to get a pending project")?;
    let pending_project = match result {
        Some(result) if result.pending_project.is_visible_to(login_user) => result.pending_project,
        _ => return Err(UseCaseError::UseCase(Error::PendingProjectNotFound)),
    };

    let result = ctx
        .get_registration_form(input.registration_form_id.into_entity())
        .await
        .context("Failed to get a registration form")?;
    let registration_form = match result {
        Some(form) if form.is_visible_to_with_pending_project(login_user, &pending_project) => form,
        _ => return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound)),
    };

    let items = interface::form_answer::to_registration_form_answer_items(
        ctx,
        &pending_project,
        &registration_form,
        input.items,
    )
    .await
    .map_err(|err| err.map_use_case(Error::from_items_error))?;

    let answer = registration_form
        .answer_by(ctx, login_user, &pending_project, items)
        .await
        .map_err(|err| UseCaseError::from_domain(err, Error::from_answer_error))?;
    ctx.store_registration_form_answer(answer.clone())
        .await
        .context("Failed to store a registration form answer")?;
    use_case_ensure!(answer.is_visible_to_with_pending_project(login_user, &pending_project));
    Ok(RegistrationFormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        pending_project::PendingProjectId, registration_form::RegistrationFormId,
        registration_form_answer::RegistrationFormAnswerRespondent,
    };
    use crate::test::interface as test_interface;
    use crate::{
        answer_registration_form, get_pending_project_registration_form_answer,
        get_registration_form_answer_shared_file, interface, UseCaseError,
    };

    use sos21_domain::test;

    #[tokio::test]
    async fn test_create_author() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let pending_project = test::model::new_general_pending_project(user.id().clone());
        let registration_form = test::model::new_registration_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let registration_form_id = RegistrationFormId::from_entity(registration_form.id);
        let pending_project_id = PendingProjectId::from_entity(pending_project.id());
        let input = answer_registration_form::Input {
            registration_form_id,
            pending_project_id,
            items: test_interface::mock_input_form_answer_items(&registration_form.items),
        };

        let got = answer_registration_form::run(&app, input).await.unwrap();
        assert!(got.registration_form_id == registration_form_id);
        assert!(
            got.respondent == RegistrationFormAnswerRespondent::PendingProject(pending_project_id)
        );

        assert!(matches!(
            get_pending_project_registration_form_answer::run(&app, pending_project_id, registration_form_id).await,
            Ok(answer)
            if answer.id == got.id
        ));
    }

    #[tokio::test]
    async fn test_create_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let pending_project = test::model::new_general_pending_project(other.id().clone());
        let registration_form = test::model::new_registration_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let registration_form_id = RegistrationFormId::from_entity(registration_form.id);
        let pending_project_id = PendingProjectId::from_entity(pending_project.id());
        let input = answer_registration_form::Input {
            registration_form_id,
            pending_project_id,
            items: test_interface::mock_input_form_answer_items(&registration_form.items),
        };

        assert!(matches!(
            answer_registration_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                answer_registration_form::Error::RegistrationFormNotFound
            ))
        ));
    }

    #[tokio::test]
    async fn test_invalid() {
        let user = test::model::new_general_user();
        let other = test::model::new_operator_user();
        let pending_project = test::model::new_general_pending_project(user.id().clone());
        let registration_form = test::model::new_registration_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let registration_form_id = RegistrationFormId::from_entity(registration_form.id);
        let pending_project_id = PendingProjectId::from_entity(pending_project.id());
        let item = test_interface::mock_input_form_answer_item(
            registration_form.items.items().next().unwrap(),
        );
        let input = answer_registration_form::Input {
            registration_form_id,
            pending_project_id,
            items: vec![item.clone(), item.clone()],
        };

        assert!(matches!(
            answer_registration_form::run(&app, input).await,
            Err(UseCaseError::UseCase(_))
        ));
    }

    #[tokio::test]
    async fn test_file_share() {
        use crate::model::{
            file::FileId, form::item::FormItemId, form_answer::item::FormAnswerItemBody,
        };

        use sos21_domain::model::form::item;

        let user = test::model::new_general_user();
        let operator = test::model::new_operator_user();
        let pending_project = test::model::new_general_pending_project(user.id().clone());

        let (registration_form, item_id) = {
            let body = item::FormItemBody::File(item::FileFormItem {
                types: None,
                accept_multiple_files: false,
                is_required: true,
            });
            let item = test::model::new_form_item_with_body(body);
            let item_id = item.id;
            let items = item::FormItems::from_items(vec![item]).unwrap();
            let registration_form =
                test::model::new_registration_form_with_items(operator.id().clone(), items);
            (registration_form, item_id)
        };
        let (file, object) = test::model::new_file(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .build();
        let user_app = app.clone().login_as(user.clone()).await;

        let answer_item = interface::form_answer::InputFormAnswerItem {
            item_id: FormItemId::from_entity(item_id),
            body: Some(interface::form_answer::InputFormAnswerItemBody::File(vec![
                interface::form_answer::InputFormAnswerItemFile::File(FileId::from_entity(file.id)),
            ])),
        };
        let registration_form_id = RegistrationFormId::from_entity(registration_form.id);
        let pending_project_id = PendingProjectId::from_entity(pending_project.id());
        let input = answer_registration_form::Input {
            registration_form_id,
            pending_project_id,
            items: vec![answer_item],
        };
        let got = answer_registration_form::run(&user_app, input)
            .await
            .unwrap();
        assert_eq!(got.registration_form_id, registration_form_id);
        assert_eq!(
            got.respondent,
            RegistrationFormAnswerRespondent::PendingProject(pending_project_id)
        );

        let answer = get_pending_project_registration_form_answer::run(
            &user_app,
            pending_project_id,
            registration_form_id,
        )
        .await
        .unwrap();
        assert_eq!(answer.id, got.id);
        let sharing_id = match &answer.items[0].body {
            Some(FormAnswerItemBody::File(sharings)) => sharings[0],
            _ => panic!("created registration form answer item is not file"),
        };

        let operator_app = app.login_as(operator.clone()).await;
        let input = get_registration_form_answer_shared_file::Input {
            answer_id: answer.id,
            sharing_id,
        };
        assert!(matches!(
            get_registration_form_answer_shared_file::run(&operator_app, input).await,
            Ok(got)
            if got.id == FileId::from_entity(file.id)
        ));
    }
}
