use crate::error::{UseCaseError, UseCaseResult};

use anyhow::Context;
use sos21_domain::context::{
    project_repository::{self, ProjectRepository},
    Login,
};
use sos21_domain::model::{permissions::Permissions, project, user};

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub field_names: InputFieldNames,
    pub category_names: InputCategoryNames,
}

#[derive(Debug, Clone)]
pub struct InputFieldNames {
    pub id: Option<String>,
    pub code: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub owner_id: Option<String>,
    pub owner_first_name: Option<String>,
    pub owner_last_name: Option<String>,
    pub owner_full_name: Option<String>,
    pub owner_kana_first_name: Option<String>,
    pub owner_kana_last_name: Option<String>,
    pub owner_kana_full_name: Option<String>,
    pub subowner_id: Option<String>,
    pub subowner_first_name: Option<String>,
    pub subowner_last_name: Option<String>,
    pub subowner_full_name: Option<String>,
    pub subowner_kana_first_name: Option<String>,
    pub subowner_kana_last_name: Option<String>,
    pub subowner_kana_full_name: Option<String>,
    pub name: Option<String>,
    pub kana_name: Option<String>,
    pub group_name: Option<String>,
    pub kana_group_name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub attribute_academic: Option<String>,
    pub attribute_artistic: Option<String>,
    pub attribute_committee: Option<String>,
    pub attribute_outdoor: Option<String>,
    pub attribute_indoor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InputCategoryNames {
    pub general: String,
    pub cooking_requiring_preparation_area: String,
    pub cooking: String,
    pub food: String,
    pub stage: String,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Vec<u8>, Error>
where
    Login<C>: ProjectRepository,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::READ_ALL_PROJECTS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let projects = ctx
        .list_projects()
        .await
        .context("Failed to list projects")?;

    // TODO: Tune buffer size and initial vector capacity
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_writer(Vec::new());

    write_header(&mut writer, &input)?;

    for project_with_owners in projects {
        let project_repository::ProjectWithOwners {
            project,
            owner,
            subowner,
        } = project_with_owners;

        use_case_ensure!(
            project.is_visible_to(login_user)
                && owner.name().is_visible_to(login_user)
                && owner.kana_name().is_visible_to(login_user)
                && subowner.name().is_visible_to(login_user)
                && subowner.kana_name().is_visible_to(login_user)
        );

        write_record(
            &mut writer,
            &input,
            WriteRecordInput {
                project,
                owner_name: owner.name(),
                owner_kana_name: owner.kana_name(),
                subowner_name: subowner.name(),
                subowner_kana_name: subowner.kana_name(),
            },
        )?;
    }

    let csv = writer.into_inner().context("Failed to write CSV data")?;
    Ok(csv)
}

// TODO: Ensure that the field orders are consistent between `write_header` and `write_record`
fn write_header<W>(writer: &mut csv::Writer<W>, input: &Input) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    let InputFieldNames {
        id,
        code,
        created_at,
        updated_at,
        owner_id,
        owner_first_name,
        owner_last_name,
        owner_full_name,
        owner_kana_first_name,
        owner_kana_last_name,
        owner_kana_full_name,
        subowner_id,
        subowner_first_name,
        subowner_last_name,
        subowner_full_name,
        subowner_kana_first_name,
        subowner_kana_last_name,
        subowner_kana_full_name,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attribute_academic,
        attribute_artistic,
        attribute_committee,
        attribute_outdoor,
        attribute_indoor,
    } = &input.field_names;

    macro_rules! write_field {
        ($writer:ident, $name:ident) => {
            if let Some(x) = $name {
                $writer.write_field(x)?;
            }
        };
    }

    write_field!(writer, id);
    write_field!(writer, code);
    write_field!(writer, created_at);
    write_field!(writer, updated_at);
    write_field!(writer, owner_id);
    write_field!(writer, owner_first_name);
    write_field!(writer, owner_last_name);
    write_field!(writer, owner_full_name);
    write_field!(writer, owner_kana_first_name);
    write_field!(writer, owner_kana_last_name);
    write_field!(writer, owner_kana_full_name);
    write_field!(writer, subowner_id);
    write_field!(writer, subowner_first_name);
    write_field!(writer, subowner_last_name);
    write_field!(writer, subowner_full_name);
    write_field!(writer, subowner_kana_first_name);
    write_field!(writer, subowner_kana_last_name);
    write_field!(writer, subowner_kana_full_name);
    write_field!(writer, name);
    write_field!(writer, kana_name);
    write_field!(writer, group_name);
    write_field!(writer, kana_group_name);
    write_field!(writer, description);
    write_field!(writer, category);
    write_field!(writer, attribute_academic);
    write_field!(writer, attribute_artistic);
    write_field!(writer, attribute_committee);
    write_field!(writer, attribute_outdoor);
    write_field!(writer, attribute_indoor);

    // this terminates the record (see docs on `csv::Writer::write_record`)
    writer.write_record(std::iter::empty::<&[u8]>())?;

    Ok(())
}

struct WriteRecordInput<'a> {
    project: project::Project,
    owner_name: &'a user::UserName,
    owner_kana_name: &'a user::UserKanaName,
    subowner_name: &'a user::UserName,
    subowner_kana_name: &'a user::UserKanaName,
}

fn write_record<W>(
    writer: &mut csv::Writer<W>,
    input: &Input,
    data: WriteRecordInput<'_>,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    let InputFieldNames {
        id,
        code,
        created_at,
        updated_at,
        owner_id,
        owner_first_name,
        owner_last_name,
        owner_full_name,
        owner_kana_first_name,
        owner_kana_last_name,
        owner_kana_full_name,
        subowner_id,
        subowner_first_name,
        subowner_last_name,
        subowner_full_name,
        subowner_kana_first_name,
        subowner_kana_last_name,
        subowner_kana_full_name,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attribute_academic,
        attribute_artistic,
        attribute_committee,
        attribute_outdoor,
        attribute_indoor,
    } = &input.field_names;

    if id.is_some() {
        writer.write_field(data.project.id().to_uuid().to_hyphenated().to_string())?;
    }

    if code.is_some() {
        writer.write_field(data.project.code().to_string())?;
    }

    if created_at.is_some() {
        let created_at = data.project.created_at().jst().format("%F %T").to_string();
        writer.write_field(created_at)?;
    }

    if updated_at.is_some() {
        let updated_at = data.project.updated_at().jst().format("%F %T").to_string();
        writer.write_field(updated_at)?;
    }

    if owner_id.is_some() {
        writer.write_field(&data.project.owner_id().0)?;
    }

    write_user_name_fields(
        writer,
        WriteUserNameFieldsInput {
            first_name: owner_first_name.as_ref(),
            last_name: owner_last_name.as_ref(),
            full_name: owner_full_name.as_ref(),
            kana_first_name: owner_kana_first_name.as_ref(),
            kana_last_name: owner_kana_last_name.as_ref(),
            kana_full_name: owner_kana_full_name.as_ref(),
        },
        data.owner_name,
        data.owner_kana_name,
    )?;

    if subowner_id.is_some() {
        writer.write_field(&data.project.subowner_id().0)?;
    }

    write_user_name_fields(
        writer,
        WriteUserNameFieldsInput {
            first_name: subowner_first_name.as_ref(),
            last_name: subowner_last_name.as_ref(),
            full_name: subowner_full_name.as_ref(),
            kana_first_name: subowner_kana_first_name.as_ref(),
            kana_last_name: subowner_kana_last_name.as_ref(),
            kana_full_name: subowner_kana_full_name.as_ref(),
        },
        data.subowner_name,
        data.subowner_kana_name,
    )?;

    if name.is_some() {
        writer.write_field(data.project.name().as_str())?;
    }

    if kana_name.is_some() {
        writer.write_field(data.project.kana_name().as_str())?;
    }

    if group_name.is_some() {
        writer.write_field(data.project.group_name().as_str())?;
    }

    if kana_group_name.is_some() {
        writer.write_field(data.project.kana_group_name().as_str())?;
    }

    if description.is_some() {
        writer.write_field(data.project.description().as_str())?;
    }

    if category.is_some() {
        let category_name = match data.project.category() {
            project::ProjectCategory::General => &input.category_names.general,
            project::ProjectCategory::CookingRequiringPreparationArea => {
                &input.category_names.cooking_requiring_preparation_area
            }
            project::ProjectCategory::Cooking => &input.category_names.cooking,
            project::ProjectCategory::Food => &input.category_names.food,
            project::ProjectCategory::Stage => &input.category_names.stage,
        };
        writer.write_field(category_name)?;
    }

    if attribute_academic.is_some() {
        if data
            .project
            .attributes()
            .contains(project::ProjectAttribute::Academic)
        {
            writer.write_field(b"TRUE")?;
        } else {
            writer.write_field(b"FALSE")?;
        }
    }

    if attribute_artistic.is_some() {
        if data
            .project
            .attributes()
            .contains(project::ProjectAttribute::Artistic)
        {
            writer.write_field(b"TRUE")?;
        } else {
            writer.write_field(b"FALSE")?;
        }
    }

    if attribute_committee.is_some() {
        if data
            .project
            .attributes()
            .contains(project::ProjectAttribute::Committee)
        {
            writer.write_field(b"TRUE")?;
        } else {
            writer.write_field(b"FALSE")?;
        }
    }

    if attribute_outdoor.is_some() {
        if data
            .project
            .attributes()
            .contains(project::ProjectAttribute::Outdoor)
        {
            writer.write_field(b"TRUE")?;
        } else {
            writer.write_field(b"FALSE")?;
        }
    }

    if attribute_indoor.is_some() {
        if data
            .project
            .attributes()
            .contains(project::ProjectAttribute::Indoor)
        {
            writer.write_field(b"TRUE")?;
        } else {
            writer.write_field(b"FALSE")?;
        }
    }

    // this terminates the record (see docs on `csv::Writer::write_record`)
    writer.write_record(std::iter::empty::<&[u8]>())?;

    Ok(())
}

struct WriteUserNameFieldsInput<'a> {
    first_name: Option<&'a String>,
    last_name: Option<&'a String>,
    full_name: Option<&'a String>,
    kana_first_name: Option<&'a String>,
    kana_last_name: Option<&'a String>,
    kana_full_name: Option<&'a String>,
}

fn write_user_name_fields<'a, W>(
    writer: &mut csv::Writer<W>,
    input: WriteUserNameFieldsInput<'_>,
    name: &'a user::UserName,
    kana_name: &'a user::UserKanaName,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    if input.first_name.is_some() {
        writer.write_field(name.first())?;
    }

    if input.last_name.is_some() {
        writer.write_field(name.last())?;
    }

    if input.full_name.is_some() {
        let first = name.first().as_bytes();
        let last = name.last().as_bytes();
        let mut full = Vec::with_capacity(first.len() + last.len() + 1);
        full.extend(last);
        full.push(b' ');
        full.extend(first);
        writer.write_field(full)?;
    }

    if input.kana_first_name.is_some() {
        writer.write_field(kana_name.first())?;
    }

    if input.kana_last_name.is_some() {
        writer.write_field(kana_name.last())?;
    }

    if input.kana_full_name.is_some() {
        let first = kana_name.first().as_bytes();
        let last = kana_name.last().as_bytes();
        let mut full = Vec::with_capacity(first.len() + last.len() + 1);
        full.extend(last);
        full.push(b' ');
        full.extend(first);
        writer.write_field(full)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{export_projects, UseCaseError};
    use sos21_domain::context::Login;
    use sos21_domain::model as domain;
    use sos21_domain::test;

    async fn prepare_app(login_user: domain::user::User) -> Login<test::context::MockApp> {
        let other = test::model::new_general_user();
        let project1 = test::model::new_general_project(login_user.id().clone());
        let project2 = test::model::new_general_project(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![login_user.clone(), other.clone()])
            .projects(vec![project1.clone(), project2.clone()])
            .build()
            .login_as(login_user.clone())
            .await;
        app
    }

    fn mock_input() -> export_projects::Input {
        let field_names = export_projects::InputFieldNames {
            id: Some("内部ID".to_string()),
            code: Some("企画番号".to_string()),
            created_at: Some("作成日時".to_string()),
            updated_at: Some("更新日時".to_string()),
            owner_id: Some("責任者".to_string()),
            owner_first_name: None,
            owner_last_name: None,
            owner_full_name: Some("責任者名".to_string()),
            owner_kana_first_name: None,
            owner_kana_last_name: None,
            owner_kana_full_name: Some("責任者名（よみがな）".to_string()),
            subowner_id: Some("副責任者".to_string()),
            subowner_first_name: None,
            subowner_last_name: None,
            subowner_full_name: Some("副責任者名".to_string()),
            subowner_kana_first_name: None,
            subowner_kana_last_name: None,
            subowner_kana_full_name: Some("副責任者名（よみがな）".to_string()),
            name: Some("企画名".to_string()),
            kana_name: Some("企画名（よみがな）".to_string()),
            group_name: Some("団体名".to_string()),
            kana_group_name: Some("団体名（よみがな）".to_string()),
            description: Some("企画説明".to_string()),
            category: Some("企画形態".to_string()),
            attribute_academic: Some("学術企画".to_string()),
            attribute_artistic: Some("芸術企画".to_string()),
            attribute_committee: Some("委員会企画".to_string()),
            attribute_outdoor: Some("屋外企画".to_string()),
            attribute_indoor: Some("屋内企画".to_string()),
        };
        let category_names = export_projects::InputCategoryNames {
            general: "一般企画（食品取扱い企画を除く）".to_string(),
            cooking_requiring_preparation_area: "調理を行う企画（仕込場が必要）".to_string(),
            cooking: "調理を行う企画（仕込場が不要）".to_string(),
            food: "飲食物取扱い企画".to_string(),
            stage: "ステージ企画".to_string(),
        };
        export_projects::Input {
            field_names,
            category_names,
        }
    }

    // Checks that the normal user cannot list projects.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let app = prepare_app(user).await;

        assert!(matches!(
            export_projects::run(&app, mock_input()).await,
            Err(UseCaseError::UseCase(
                export_projects::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can list projects.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let app = prepare_app(user).await;

        assert!(matches!(
            export_projects::run(&app, mock_input()).await,
            Ok(_)
        ));
    }

    // Checks that the privileged committee user can list projects.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let app = prepare_app(user).await;

        assert!(matches!(
            export_projects::run(&app, mock_input()).await,
            Ok(_)
        ));
    }
}
