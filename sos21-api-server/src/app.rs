use crate::config::Config;

use anyhow::Result;
use sos21_gateway_database::Database;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Clone)]
pub struct App {
    database: Database,
    config: Config,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_database_connections)
            .connect(&config.postgres_uri)
            .await?;
        let database = Database::new(pool);
        Ok(App { database, config })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

sos21_domain_context::delegate_user_repository! { impl<> for App : database }
sos21_domain_context::delegate_project_repository! { impl<> for App : database }
