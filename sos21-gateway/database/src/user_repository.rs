use anyhow::Result;
use futures::{future, stream::TryStreamExt};
use sos21_database::{command, model as data, query};
use sos21_domain_context::UserRepository;
use sos21_domain_model::{
    email::EmailAddress,
    user::{User, UserId, UserKanaName, UserName, UserRole},
};
use sqlx::postgres::PgPool;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Database { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for Database {
    async fn store_user(&self, user: User) -> Result<()> {
        let mut transaction = self.pool.begin().await?;

        let user = from_user(user);
        if query::find_user(&mut transaction, user.id.clone())
            .await?
            .is_some()
        {
            let input = command::update_user::Input {
                id: user.id,
                first_name: user.first_name,
                kana_first_name: user.kana_first_name,
                last_name: user.last_name,
                kana_last_name: user.kana_last_name,
                role: user.role,
            };
            command::update_user(&mut transaction, input).await?;
        } else {
            command::insert_user(&mut transaction, user).await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        query::find_user(&self.pool, id.0)
            .await
            .and_then(|opt| opt.map(to_user).transpose())
    }

    async fn list_users(&self) -> Result<Vec<User>> {
        query::list_users(&self.pool)
            .and_then(|user| future::ready(to_user(user)))
            .try_collect()
            .await
    }
}

fn from_user(user: User) -> data::user::User {
    let User {
        id,
        created_at,
        name,
        kana_name,
        email,
        role,
    } = user;
    let (first_name, last_name) = name.into_string();
    let (kana_first_name, kana_last_name) = kana_name.into_string();
    data::user::User {
        id: id.0,
        created_at,
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        email: email.into_string(),
        role: match role {
            UserRole::Administrator => data::user::UserRole::Administrator,
            UserRole::CommitteeOperator => data::user::UserRole::CommitteeOperator,
            UserRole::Committee => data::user::UserRole::Committee,
            UserRole::General => data::user::UserRole::General,
        },
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
        role,
    } = user;
    Ok(User {
        id: UserId(id),
        created_at,
        name: UserName::from_string(first_name, last_name)?,
        kana_name: UserKanaName::from_string(kana_first_name, kana_last_name)?,
        email: EmailAddress::from_string(email)?,
        role: match role {
            data::user::UserRole::Administrator => UserRole::Administrator,
            data::user::UserRole::CommitteeOperator => UserRole::CommitteeOperator,
            data::user::UserRole::Committee => UserRole::Committee,
            data::user::UserRole::General => UserRole::General,
        },
    })
}
