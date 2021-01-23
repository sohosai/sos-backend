use std::convert::TryInto;

use crate::config::Config;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sos21_context::{TimeContext, UserRepository};
use sos21_database::{command, model as data, query};
use sos21_model::{
    email::EmailAddress,
    role::Role,
    user::{User, UserId, UserName},
};
use sqlx::postgres::{PgPool, PgPoolOptions};

mod authentication;
pub use authentication::Authentication;

#[derive(Debug, Clone)]
pub struct App {
    pub pool: PgPool,
    pub config: Config,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_database_connections)
            .connect(&config.postgres_uri)
            .await?;
        Ok(App { pool, config })
    }
}

#[async_trait::async_trait]
impl UserRepository for App {
    async fn create_user(&self, user: User) -> Result<()> {
        let User {
            id,
            created_at,
            name,
            email,
            role,
        } = user;
        let UserName { first, last } = name;
        let user = data::user::User {
            id: id.into(),
            created_at,
            first_name: first.into(),
            last_name: last.into(),
            email: email.into(),
            role: match role {
                Role::Administrator => data::user::UserRole::Administrator,
                Role::Committee => data::user::UserRole::Committee,
                Role::General => data::user::UserRole::General,
            },
        };
        command::insert_user(&self.pool, user).await
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        if let Some(user) = query::find_user(&self.pool, id.into()).await? {
            let data::user::User {
                id,
                created_at,
                first_name,
                last_name,
                email,
                role,
            } = user;
            Ok(Some(User {
                id: UserId(id),
                created_at,
                name: UserName {
                    first: first_name.try_into()?,
                    last: last_name.try_into()?,
                },
                email: EmailAddress::from_string(email)?,
                role: match role {
                    data::user::UserRole::Administrator => Role::Administrator,
                    data::user::UserRole::Committee => Role::Committee,
                    data::user::UserRole::General => Role::General,
                },
            }))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl TimeContext for App {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
