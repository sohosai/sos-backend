use std::fmt::{self, Debug};

use crate::error::{UseCaseError, UseCaseResult};
use crate::model::registration_form::RegistrationFormId;

use anyhow::{bail, Context};
use sos21_domain::context::{Login, RegistrationFormAnswerRepository, RegistrationFormRepository};
use sos21_domain::model::{
    form, form_answer, permissions, registration_form, registration_form_answer, user,
};

#[derive(Debug, Clone)]
pub enum Error {
    RegistrationFormNotFound,
    InsufficientPermissions,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[derive(Debug, Clone)]
pub struct RenderFileAnswerInput {
    pub answer_id: String,
    pub sharing_ids: Vec<String>,
}

pub struct Input<F> {
    pub registration_form_id: RegistrationFormId,
    pub field_names: InputFieldNames,
    pub render_file_answer: F,
}

impl<F> Debug for Input<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Input")
            .field("registration_form_id", &self.registration_form_id)
            .field("field_names", &self.field_names)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct InputFieldNames {
    pub id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub project_id: Option<String>,
    pub pending_project_id: Option<String>,
    pub author_id: Option<String>,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C, F>(ctx: &Login<C>, input: Input<F>) -> UseCaseResult<Vec<u8>, Error>
where
    C: RegistrationFormRepository + RegistrationFormAnswerRepository + Send + Sync,
    F: Fn(RenderFileAnswerInput) -> anyhow::Result<String> + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(permissions::Permissions::READ_ALL_REGISTRATION_FORM_ANSWERS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let result = ctx
        .get_registration_form(input.registration_form_id.into_entity())
        .await
        .context("Failed to get a registration form")?;
    let registration_form = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound)),
    };

    let answers = ctx
        .list_registration_form_answers(registration_form.id)
        .await
        .context("Failed to list registration form answers")?;

    // TODO: Tune buffer size and initial vector capacity
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_writer(Vec::new());

    write_header(&mut writer, &input.field_names, &registration_form)?;

    for answer in answers {
        use_case_ensure!(answer.is_visible_to(login_user));
        write_record(&mut writer, &input, &registration_form, answer)?;
    }

    let csv = writer.into_inner().context("Failed to write CSV data")?;
    Ok(csv)
}

// TODO: Ensure that the field orders are consistent between `write_header` and `write_record`
fn write_header<W>(
    writer: &mut csv::Writer<W>,
    field_names: &InputFieldNames,
    registration_form: &registration_form::RegistrationForm,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    let InputFieldNames {
        id,
        created_at,
        updated_at,
        project_id,
        pending_project_id,
        author_id,
    } = field_names;

    macro_rules! write_field {
        ($writer:ident, $name:ident) => {
            if let Some(x) = $name {
                $writer.write_field(x)?;
            }
        };
    }

    write_field!(writer, id);
    write_field!(writer, created_at);
    write_field!(writer, updated_at);
    write_field!(writer, project_id);
    write_field!(writer, pending_project_id);
    write_field!(writer, author_id);

    for item in registration_form.items.items() {
        write_item_header_fields(writer, item)?;
    }

    // this terminates the record (see docs on `csv::Writer::write_record`)
    writer.write_record(std::iter::empty::<&[u8]>())?;

    Ok(())
}

fn write_item_header_fields<W>(
    writer: &mut csv::Writer<W>,
    item: &form::item::FormItem,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    use form::item::FormItemBody;

    match &item.body {
        FormItemBody::Checkbox(checkbox_item) => {
            for checkbox in checkbox_item.boxes() {
                let mut field_name = item.name.as_str().to_string();
                field_name.push(' ');
                field_name.push_str(checkbox.label.as_str());
                writer.write_field(field_name)?;
            }
        }
        FormItemBody::GridRadio(grid_item) => {
            for row in grid_item.rows() {
                let mut field_name = item.name.as_str().to_string();
                field_name.push(' ');
                field_name.push_str(row.label.as_str());
                writer.write_field(field_name)?;
            }
        }
        _ => {
            writer.write_field(item.name.as_str())?;
        }
    }

    Ok(())
}

fn write_record<W, F>(
    writer: &mut csv::Writer<W>,
    input: &Input<F>,
    registration_form: &registration_form::RegistrationForm,
    answer: registration_form_answer::RegistrationFormAnswer,
) -> anyhow::Result<()>
where
    W: std::io::Write,
    F: Fn(RenderFileAnswerInput) -> anyhow::Result<String>,
{
    let InputFieldNames {
        id,
        created_at,
        updated_at,
        project_id,
        pending_project_id,
        author_id,
    } = &input.field_names;

    if id.is_some() {
        writer.write_field(answer.id().to_uuid().to_hyphenated().to_string())?;
    }

    if created_at.is_some() {
        let created_at = answer.created_at().jst().format("%F %T").to_string();
        writer.write_field(created_at)?;
    }

    if updated_at.is_some() {
        let updated_at = answer.updated_at().jst().format("%F %T").to_string();
        writer.write_field(updated_at)?;
    }

    if project_id.is_some() {
        if let registration_form_answer::RegistrationFormAnswerRespondent::Project(project_id) =
            answer.respondent()
        {
            writer.write_field(project_id.to_uuid().to_hyphenated().to_string())?;
        } else {
            writer.write_field("")?;
        }
    }

    if pending_project_id.is_some() {
        if let registration_form_answer::RegistrationFormAnswerRespondent::PendingProject(
            pending_project_id,
        ) = answer.respondent()
        {
            writer.write_field(pending_project_id.to_uuid().to_hyphenated().to_string())?;
        } else {
            writer.write_field("")?;
        }
    }

    if author_id.is_some() {
        writer.write_field(&answer.author_id().0)?;
    }

    let answer_id = answer.id().to_uuid().to_hyphenated().to_string();
    let render = |sharing_ids| {
        (input.render_file_answer)(RenderFileAnswerInput {
            answer_id: answer_id.clone(),
            sharing_ids,
        })
    };
    for (item, answer_item) in registration_form
        .items
        .items()
        .zip(answer.into_items().into_items())
    {
        write_item_fields(writer, render, item, answer_item)?;
    }

    // this terminates the record (see docs on `csv::Writer::write_record`)
    writer.write_record(std::iter::empty::<&[u8]>())?;

    Ok(())
}

fn write_item_fields<W, F>(
    writer: &mut csv::Writer<W>,
    render_file_answer: F,
    item: &form::item::FormItem,
    answer_item: form_answer::item::FormAnswerItem,
) -> anyhow::Result<()>
where
    W: std::io::Write,
    F: FnOnce(Vec<String>) -> anyhow::Result<String>,
{
    let body = match answer_item.body {
        Some(body) => body,
        None => {
            writer.write_field("")?;
            return Ok(());
        }
    };

    use form::item::FormItemBody;
    use form_answer::item::FormAnswerItemBody;
    match body {
        FormAnswerItemBody::Text(None) => writer.write_field("")?,
        FormAnswerItemBody::Text(Some(answer)) => writer.write_field(answer.into_string())?,
        FormAnswerItemBody::Integer(None) => writer.write_field("")?,
        FormAnswerItemBody::Integer(Some(answer)) => writer.write_field(answer.to_string())?,
        FormAnswerItemBody::Checkbox(checks) => {
            let item = match &item.body {
                FormItemBody::Checkbox(item) => item,
                _ => bail!("unexpectedly mismatched form item and form answer item"),
            };

            for checkbox in item.boxes() {
                if checks.is_checked(checkbox.id) {
                    writer.write_field(b"TRUE")?;
                } else {
                    writer.write_field(b"FALSE")?;
                }
            }
        }
        FormAnswerItemBody::Radio(None) => writer.write_field("")?,
        FormAnswerItemBody::Radio(Some(answer_id)) => {
            let item = match &item.body {
                FormItemBody::Radio(item) => item,
                _ => bail!("unexpectedly mismatched form item and form answer item"),
            };

            let button = match item.buttons.buttons().find(|button| button.id == answer_id) {
                Some(button) => button,
                None => bail!("unexpectedly unknown radio id in the answer"),
            };

            writer.write_field(button.label.as_str())?;
        }
        FormAnswerItemBody::GridRadio(rows) => {
            let item = match &item.body {
                FormItemBody::GridRadio(item) => item,
                _ => bail!("unexpectedly mismatched form item and form answer item"),
            };

            for row_answer in rows.row_answers() {
                let column_id = match row_answer.value {
                    Some(column_id) => column_id,
                    None => {
                        writer.write_field("")?;
                        continue;
                    }
                };

                let column = match item.columns().find(|column| column.id == column_id) {
                    Some(column) => column,
                    None => bail!("unexpectedly unknown column id in the answer"),
                };

                writer.write_field(column.label.as_str())?;
            }
        }
        FormAnswerItemBody::File(sharings) => {
            let sharings = sharings
                .sharing_answers()
                .map(|answer| answer.sharing_id.to_uuid().to_hyphenated().to_string())
                .collect();
            let field = (render_file_answer)(sharings).context("Failed to render file answer")?;
            writer.write_field(field)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::model::registration_form::RegistrationFormId;
    use crate::{export_registration_form_answers, UseCaseError};
    use sos21_domain::context::Login;
    use sos21_domain::model as domain;
    use sos21_domain::test;

    async fn prepare_app(
        login_user: domain::user::User,
    ) -> (Login<test::context::MockApp>, RegistrationFormId) {
        let operator = test::model::new_operator_user();

        let project = test::model::new_general_project(login_user.id().clone());
        let pending_project = test::model::new_general_pending_project(login_user.id().clone());

        let registration_form1 = test::model::new_registration_form(operator.id().clone());
        let registration_form1_id = RegistrationFormId::from_entity(registration_form1.id);
        let registration_form1_answer1 = test::model::new_registration_form_answer_with_project(
            login_user.id().clone(),
            project.id(),
            &registration_form1,
        );
        let registration_form1_answer2 =
            test::model::new_registration_form_answer_with_pending_project(
                login_user.id().clone(),
                pending_project.id(),
                &registration_form1,
            );

        let app = test::build_mock_app()
            .users(vec![login_user.clone(), operator])
            .registration_forms(vec![registration_form1])
            .projects(vec![project])
            .pending_projects(vec![pending_project])
            .registration_form_answers(vec![registration_form1_answer1, registration_form1_answer2])
            .build()
            .login_as(login_user.clone())
            .await;
        (app, registration_form1_id)
    }

    fn mock_input(
        registration_form_id: RegistrationFormId,
    ) -> export_registration_form_answers::Input<
        impl Fn(export_registration_form_answers::RenderFileAnswerInput) -> anyhow::Result<String>,
    > {
        let field_names = export_registration_form_answers::InputFieldNames {
            id: None,
            created_at: Some("作成日時".to_string()),
            updated_at: Some("更新日時".to_string()),
            project_id: Some("企画番号".to_string()),
            pending_project_id: Some("承認待ち企画番号".to_string()),
            author_id: Some("回答者".to_string()),
        };
        let render_file_answer =
            |input: export_registration_form_answers::RenderFileAnswerInput| {
                Ok(format!(
                    "{},{}",
                    input.answer_id,
                    input.sharing_ids.join(",")
                ))
            };
        export_registration_form_answers::Input {
            registration_form_id,
            field_names,
            render_file_answer,
        }
    }

    // Checks that the normal user cannot export registration_form answers.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let (app, registration_form_id) = prepare_app(user).await;
        let input = mock_input(registration_form_id);

        assert!(matches!(
            export_registration_form_answers::run(&app, input).await,
            Err(UseCaseError::UseCase(
                export_registration_form_answers::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can export registration_form answers.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let (app, registration_form_id) = prepare_app(user).await;

        assert!(matches!(
            export_registration_form_answers::run(&app, mock_input(registration_form_id)).await,
            Ok(_)
        ));
    }

    // Checks that the privileged committee user can export registration_form answers.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let (app, registration_form_id) = prepare_app(user).await;

        assert!(matches!(
            export_registration_form_answers::run(&app, mock_input(registration_form_id)).await,
            Ok(_)
        ));
    }
}
