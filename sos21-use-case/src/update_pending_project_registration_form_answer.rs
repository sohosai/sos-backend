use crate::error::{UseCaseError, UseCaseResult};
use crate::interface;
use crate::model::form::FormItemId;
use crate::model::pending_project::PendingProjectId;
use crate::model::registration_form::RegistrationFormId;
use crate::model::registration_form_answer::RegistrationFormAnswer;

use anyhow::Context;
use sos21_domain::context::{
    ConfigContext, FileRepository, FileSharingRepository, Login, PendingProjectRepository,
    RegistrationFormAnswerRepository, RegistrationFormRepository,
};
use sos21_domain::model::{date_time, registration_form_answer};

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
    RegistrationFormAnswerNotFound,
    OutOfProjectCreationPeriod,
    InvalidItems(interface::form_answer::FormAnswerItemsError),
    InvalidAnswer(interface::form::CheckAnswerError),
    InsufficientPermissions,
}

impl Error {
    fn from_items_error(err: interface::form_answer::FormAnswerItemsError) -> Self {
        Error::InvalidItems(err)
    }

    fn from_set_error(err: registration_form_answer::SetItemsError) -> Self {
        match err.kind() {
            registration_form_answer::SetItemsErrorKind::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
            registration_form_answer::SetItemsErrorKind::MismatchedItemsLength => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::MismatchedItemsLength)
            }
            registration_form_answer::SetItemsErrorKind::MismatchedItemId { expected, got } => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::MismatchedItemId {
                    expected: FormItemId::from_entity(expected),
                    got: FormItemId::from_entity(got),
                })
            }
            registration_form_answer::SetItemsErrorKind::InvalidItem { id, kind } => {
                Error::InvalidAnswer(interface::form::CheckAnswerError::InvalidAnswerItem {
                    item_id: FormItemId::from_entity(id),
                    item_error: interface::form::to_check_answer_item_error(kind),
                })
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
        Some(registration_form)
            if registration_form
                .is_visible_to_with_pending_project(login_user, &pending_project) =>
        {
            registration_form
        }
        _ => return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound)),
    };

    let result = ctx
        .get_registration_form_answer_by_registration_form_and_pending_project(
            registration_form.id(),
            pending_project.id(),
        )
        .await
        .context("Failed to get a registration form answer")?;
    let mut answer = match result {
        Some(answer) if answer.is_visible_to_with_pending_project(login_user, &pending_project) => {
            answer
        }
        _ => return Err(UseCaseError::UseCase(Error::RegistrationFormAnswerNotFound)),
    };

    if !ctx
        .project_creation_period_for(pending_project.category())
        .contains(date_time::DateTime::now())
    {
        return Err(UseCaseError::UseCase(Error::OutOfProjectCreationPeriod));
    }

    let items = interface::form_answer::to_registration_form_answer_items(
        ctx,
        &pending_project,
        &registration_form,
        input.items,
    )
    .await
    .map_err(|err| err.map_use_case(Error::from_items_error))?;

    answer
        .set_items_with_pending_project(
            ctx,
            login_user,
            &registration_form,
            &pending_project,
            items,
        )
        .map_err(|err| UseCaseError::from_domain(err, Error::from_set_error))?;

    ctx.store_registration_form_answer(answer.clone())
        .await
        .context("Failed to store a registration form answer")?;
    use_case_ensure!(answer.is_visible_to_with_pending_project(login_user, &pending_project));
    Ok(RegistrationFormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        form::item::FormItemId, form_answer::item::FormAnswerItemBody,
        pending_project::PendingProjectId, registration_form::RegistrationFormId,
    };
    use crate::{interface, update_pending_project_registration_form_answer, UseCaseError};

    use sos21_domain::model::{
        form::item, form_answer::item as answer_item, registration_form_answer,
    };
    use sos21_domain::test;

    fn prepare_items() -> (
        item::FormItemId,
        item::FormItems,
        answer_item::FormAnswerItems,
    ) {
        let body = item::FormItemBody::Integer(
            item::IntegerFormItem::from_content(item::integer::IntegerFormItemContent {
                is_required: false,
                max: None,
                min: None,
                placeholder: None,
                unit: None,
            })
            .unwrap(),
        );
        let item = test::model::new_form_item_with_body(body);
        let item_id = item.id;
        let items = item::FormItems::from_items(vec![item]).unwrap();
        let answer_item = answer_item::FormAnswerItem {
            item_id,
            body: Some(answer_item::FormAnswerItemBody::Integer(Some(10))),
        };
        let answer_items = answer_item::FormAnswerItems::from_items(vec![answer_item]).unwrap();
        (item_id, items, answer_items)
    }

    #[tokio::test]
    async fn test_answer_in_period_general_owner() {
        let owner = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(owner.id().clone());
        let operator = test::model::new_operator_user();

        let (item_id, items, answer_items) = prepare_items();

        let item_id = FormItemId::from_entity(item_id);
        let registration_form =
            test::model::new_registration_form_with_items(operator.id().clone(), items);
        let answer = test::model::new_registration_form_answer_with_items(
            owner.id().clone(),
            registration_form_answer::RegistrationFormAnswerRespondent::PendingProject(
                pending_project.id(),
            ),
            &registration_form,
            answer_items,
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer.clone()])
            .build()
            .login_as(owner)
            .await;

        let answer_item = interface::form_answer::InputFormAnswerItem {
            item_id,
            body: Some(interface::form_answer::InputFormAnswerItemBody::Integer(
                Some(20),
            )),
        };
        let input = update_pending_project_registration_form_answer::Input {
            pending_project_id: PendingProjectId::from_entity(pending_project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id()),
            items: vec![answer_item],
        };
        let answer = update_pending_project_registration_form_answer::run(&app, input)
            .await
            .unwrap();

        assert_eq!(answer.items[0].item_id, item_id);
        assert_eq!(
            answer.items[0].body,
            Some(FormAnswerItemBody::Integer(Some(20)))
        );
    }

    #[tokio::test]
    async fn test_answer_in_period_general_other() {
        let owner = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(owner.id().clone());
        let operator = test::model::new_operator_user();

        let (item_id, items, answer_items) = prepare_items();

        let item_id = FormItemId::from_entity(item_id);
        let registration_form =
            test::model::new_registration_form_with_items(operator.id().clone(), items);
        let answer = test::model::new_registration_form_answer_with_items(
            owner.id().clone(),
            registration_form_answer::RegistrationFormAnswerRespondent::PendingProject(
                pending_project.id(),
            ),
            &registration_form,
            answer_items,
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer.clone()])
            .build()
            .login_as(other)
            .await;

        let answer_item = interface::form_answer::InputFormAnswerItem {
            item_id,
            body: Some(interface::form_answer::InputFormAnswerItemBody::Integer(
                Some(20),
            )),
        };
        let input = update_pending_project_registration_form_answer::Input {
            pending_project_id: PendingProjectId::from_entity(pending_project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id()),
            items: vec![answer_item],
        };
        assert!(matches!(
            update_pending_project_registration_form_answer::run(&app, input).await,
            Err(UseCaseError::UseCase(_))
        ));
    }

    #[tokio::test]
    async fn test_answer_after_period_owner() {
        let owner = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(owner.id().clone());
        let operator = test::model::new_operator_user();

        let (_, items, answer_items) = prepare_items();

        let registration_form =
            test::model::new_registration_form_with_items(operator.id().clone(), items);
        let answer = test::model::new_registration_form_answer_with_items(
            owner.id().clone(),
            registration_form_answer::RegistrationFormAnswerRespondent::PendingProject(
                pending_project.id(),
            ),
            &registration_form,
            answer_items,
        );

        let period = test::model::new_project_creation_period_to_now();
        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer.clone()])
            .project_creation_period_for(pending_project.category(), period)
            .build()
            .login_as(owner)
            .await;

        let input = update_pending_project_registration_form_answer::Input {
            pending_project_id: PendingProjectId::from_entity(pending_project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id()),
            items: vec![],
        };
        assert!(matches!(
            update_pending_project_registration_form_answer::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_pending_project_registration_form_answer::Error::OutOfProjectCreationPeriod
            ))
        ));
    }

    #[tokio::test]
    async fn test_answer_in_period_invalid() {
        let owner = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(owner.id().clone());
        let operator = test::model::new_operator_user();

        let (_, items, answer_items) = prepare_items();

        let registration_form =
            test::model::new_registration_form_with_items(operator.id().clone(), items);
        let answer = test::model::new_registration_form_answer_with_items(
            owner.id().clone(),
            registration_form_answer::RegistrationFormAnswerRespondent::PendingProject(
                pending_project.id(),
            ),
            &registration_form,
            answer_items,
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), operator.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer.clone()])
            .build()
            .login_as(owner)
            .await;

        let input = update_pending_project_registration_form_answer::Input {
            pending_project_id: PendingProjectId::from_entity(pending_project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id()),
            items: vec![],
        };
        assert!(matches!(
            update_pending_project_registration_form_answer::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_pending_project_registration_form_answer::Error::InvalidItems(..)
            ))
        ));
    }
}
