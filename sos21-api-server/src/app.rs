use crate::config::Config;

use anyhow::{Context as _, Result};
use sos21_gateway_database::Database;
use sqlx::{
    pool::PoolConnection,
    postgres::{PgPool, PgPoolOptions, Postgres},
};

#[derive(Debug, Clone)]
pub struct App {
    pool: PgPool,
    config: Config,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_database_connections)
            .connect(&config.postgres_uri)
            .await?;
        Ok(App { pool, config })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn connection(&self) -> Result<PoolConnection<Postgres>> {
        self.pool
            .acquire()
            .await
            .context("Failed to acquire a connection from pool")
    }

    pub async fn start_context(&self) -> Result<Context> {
        let connection = self
            .pool
            .begin()
            .await
            .context("Failed to acquire a connection from pool")?;
        let database = Database::new(connection);
        Ok(Context { database })
    }
}

#[derive(Debug)]
pub struct Context {
    database: Database,
}

impl Context {
    pub async fn commit_changes(self) -> Result<()> {
        self.database.into_connection().commit().await?;
        Ok(())
    }
}

sos21_domain::delegate_user_repository! {
    impl UserRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_project_repository! {
    impl ProjectRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_form_repository! {
    impl FormRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_form_answer_repository! {
    impl FormAnswerRepository for Context {
        self { &self.database }
    }
}
