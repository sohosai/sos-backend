use crate::error::{UseCaseError, UseCaseResult};

use anyhow::Context;
use sos21_domain::context::{Login, UserRepository};
use sos21_domain::model::{permissions::Permissions, user};

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub field_names: InputFieldNames,
    pub role_names: InputRoleNames,
    pub category_names: InputCategoryNames,
}

#[derive(Debug, Clone)]
pub struct InputFieldNames {
    pub id: Option<String>,
    pub created_at: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub kana_first_name: Option<String>,
    pub kana_last_name: Option<String>,
    pub kana_full_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub affiliation: Option<String>,
    pub role: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InputRoleNames {
    pub administrator: String,
    pub committee_operator: String,
    pub committee: String,
    pub general: String,
}

#[derive(Debug, Clone)]
pub struct InputCategoryNames {
    pub undergraduate_student: String,
    pub graduate_student: String,
    pub academic_staff: String,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Vec<u8>, Error>
where
    Login<C>: UserRepository,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::READ_ALL_USERS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let users = ctx.list_users().await.context("Failed to list users")?;
    use_case_ensure!(users.iter().all(|user| user.is_visible_to(login_user)));

    // TODO: Tune buffer size and initial vector capacity
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_writer(Vec::new());

    write_header(&mut writer, &input)?;

    for user in users {
        write_record(&mut writer, &input, user)?;
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
        created_at,
        first_name,
        last_name,
        full_name,
        kana_first_name,
        kana_last_name,
        kana_full_name,
        email,
        phone_number,
        affiliation,
        role,
        category,
    } = &input.field_names;

    macro_rules! write_field {
        ($writer:ident, $name:ident) => {
            if let Some(x) = $name {
                $writer.write_field(x)?;
            }
        };
    }

    write_field!(writer, id);
    write_field!(writer, created_at);
    write_field!(writer, first_name);
    write_field!(writer, last_name);
    write_field!(writer, full_name);
    write_field!(writer, kana_first_name);
    write_field!(writer, kana_last_name);
    write_field!(writer, kana_full_name);
    write_field!(writer, email);
    write_field!(writer, phone_number);
    write_field!(writer, affiliation);
    write_field!(writer, role);
    write_field!(writer, category);

    // this terminates the record (see docs on `csv::Writer::write_record`)
    writer.write_record(std::iter::empty::<&[u8]>())?;

    Ok(())
}

fn write_record<W>(
    writer: &mut csv::Writer<W>,
    input: &Input,
    user: user::User,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    let InputFieldNames {
        id,
        created_at,
        first_name,
        last_name,
        full_name,
        kana_first_name,
        kana_last_name,
        kana_full_name,
        email,
        phone_number,
        affiliation,
        role,
        category,
    } = &input.field_names;

    if id.is_some() {
        writer.write_field(user.id.0)?;
    }

    if created_at.is_some() {
        let created_at = user.created_at.jst().format("%F %T").to_string();
        writer.write_field(created_at)?;
    }

    if first_name.is_some() {
        writer.write_field(user.name.first())?;
    }

    if last_name.is_some() {
        writer.write_field(user.name.last())?;
    }

    if full_name.is_some() {
        let first = user.name.first().as_bytes();
        let last = user.name.last().as_bytes();
        let mut full = Vec::with_capacity(first.len() + last.len() + 1);
        full.extend(last);
        full.push(b' ');
        full.extend(first);
        writer.write_field(full)?;
    }

    if kana_first_name.is_some() {
        writer.write_field(user.kana_name.first())?;
    }

    if kana_last_name.is_some() {
        writer.write_field(user.kana_name.last())?;
    }

    if kana_full_name.is_some() {
        let first = user.kana_name.first().as_bytes();
        let last = user.kana_name.last().as_bytes();
        let mut full = Vec::with_capacity(first.len() + last.len() + 1);
        full.extend(last);
        full.push(b' ');
        full.extend(first);
        writer.write_field(full)?;
    }

    if email.is_some() {
        writer.write_field(user.email.into_string())?;
    }

    if phone_number.is_some() {
        let phone_number = user.phone_number.into_string();

        let mut raw = Vec::with_capacity(phone_number.len() + 3);
        raw.extend(b"=\"");
        if let Some(rest) = phone_number.strip_prefix("+81") {
            raw.push(b'0');
            raw.extend(rest.as_bytes());
        } else {
            raw.extend(phone_number.as_bytes());
        }
        raw.push(b'"');

        writer.write_field(raw)?;
    }

    if affiliation.is_some() {
        writer.write_field(user.affiliation.into_string())?;
    }

    if role.is_some() {
        let role_name = match user.role {
            user::UserRole::Administrator => &input.role_names.administrator,
            user::UserRole::CommitteeOperator => &input.role_names.committee_operator,
            user::UserRole::Committee => &input.role_names.committee,
            user::UserRole::General => &input.role_names.general,
        };
        writer.write_field(role_name)?;
    }

    if category.is_some() {
        let category_name = match user.category {
            user::UserCategory::UndergraduateStudent => &input.category_names.undergraduate_student,
            user::UserCategory::GraduateStudent => &input.category_names.graduate_student,
            user::UserCategory::AcademicStaff => &input.category_names.academic_staff,
        };
        writer.write_field(category_name)?;
    }

    // this terminates the record (see docs on `csv::Writer::write_record`)
    writer.write_record(std::iter::empty::<&[u8]>())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{export_users, UseCaseError};
    use sos21_domain::test;

    fn mock_input() -> export_users::Input {
        let field_names = export_users::InputFieldNames {
            id: Some("ID".to_string()),
            created_at: Some("作成日時".to_string()),
            first_name: None,
            last_name: None,
            full_name: Some("名前".to_string()),
            kana_first_name: None,
            kana_last_name: None,
            kana_full_name: Some("名前（よみがな）".to_string()),
            email: Some("メールアドレス".to_string()),
            phone_number: Some("電話番号".to_string()),
            affiliation: None,
            role: Some("権限".to_string()),
            category: Some("分類".to_string()),
        };
        let role_names = export_users::InputRoleNames {
            administrator: "管理者".to_string(),
            committee_operator: "実委人（権限持ち）".to_string(),
            committee: "実委人".to_string(),
            general: "一般".to_string(),
        };
        let category_names = export_users::InputCategoryNames {
            undergraduate_student: "学部生".to_string(),
            graduate_student: "院生".to_string(),
            academic_staff: "教職員".to_string(),
        };
        export_users::Input {
            field_names,
            role_names,
            category_names,
        }
    }

    // Checks that the normal user cannot list users.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            export_users::run(&app, mock_input()).await,
            Err(UseCaseError::UseCase(
                export_users::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot list users.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            export_users::run(&app, mock_input()).await,
            Err(UseCaseError::UseCase(
                export_users::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user can list users.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(export_users::run(&app, mock_input()).await, Ok(_)));
    }
}
