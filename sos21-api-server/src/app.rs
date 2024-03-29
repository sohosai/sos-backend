use std::collections::HashMap;
use std::fmt::{self, Debug};

use crate::config::Config;

use anyhow::{Context as _, Result};
use chrono::{TimeZone, Utc};
use rusoto_s3::S3Client;
use sos21_domain::model::{
    date_time::DateTime, project::ProjectCategory, project_creation_period::ProjectCreationPeriod,
    user::UserEmailAddress,
};
use sos21_gateway_database::Database;
use sos21_gateway_s3::S3;
use sqlx::{
    pool::PoolConnection,
    postgres::{PgPool, PgPoolOptions, Postgres},
};

#[derive(Clone)]
pub struct App {
    pool: PgPool,
    s3_client: S3Client,
    config: Config,
    administrator_email: UserEmailAddress,
    project_creation_periods: HashMap<ProjectCategory, ProjectCreationPeriod>,
}

impl Debug for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `S3Client` does't implement `Debug`,
        // so using an unit struct below as a replacement.
        #[derive(Debug)]
        struct S3Client;

        f.debug_struct("App")
            .field("pool", &self.pool)
            .field("s3_client", &S3Client)
            .field("config", &self.config)
            .finish()
    }
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_database_connections)
            .connect(&config.postgres_uri)
            .await?;

        let dispatcher = rusoto_core::request::HttpClient::new()?;
        let credentials = rusoto_credential::StaticProvider::new_minimal(
            config.s3_access_key.clone(),
            config.s3_access_secret.clone(),
        );
        let region = rusoto_core::Region::Custom {
            name: config.s3_region.clone(),
            endpoint: config.s3_endpoint.clone(),
        };
        let s3_client = S3Client::new_with(dispatcher, credentials, region);

        let administrator_email = UserEmailAddress::from_string(config.administrator_email.clone())
            .context("invalid administrator email")?;

        let mut project_creation_periods = HashMap::new();
        for (category, period) in &config.project_creation_periods {
            let category = category.parse()?;
            let period = match period.as_ref() {
                "always" => ProjectCreationPeriod::always(),
                "never" => ProjectCreationPeriod::never(),
                _ => {
                    let (starts_at, ends_at) = period
                        .split_once('-')
                        .context("period must be delimited with '-'")?;
                    // FIXME: 範囲外の場合にハンドリングする
                    let starts_at =
                        DateTime::from_utc(Utc.timestamp_millis_opt(starts_at.parse()?).unwrap());
                    let ends_at =
                        DateTime::from_utc(Utc.timestamp_millis_opt(ends_at.parse()?).unwrap());
                    ProjectCreationPeriod::from_datetime(starts_at, ends_at)
                        .context("invalid project creation period")?
                }
            };
            project_creation_periods.insert(category, period);
        }

        Ok(App {
            pool,
            s3_client,
            config,
            administrator_email,
            project_creation_periods,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn s3_client(&self) -> &S3Client {
        &self.s3_client
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
        let s3 = S3::new(self.s3_client.clone(), self.config.s3_object_bucket.clone());
        Ok(Context {
            database,
            s3,
            administrator_email: self.administrator_email.clone(),
            project_creation_periods: self.project_creation_periods.clone(),
        })
    }
}

#[derive(Debug)]
pub struct Context {
    database: Database,
    s3: S3,
    administrator_email: UserEmailAddress,
    project_creation_periods: HashMap<ProjectCategory, ProjectCreationPeriod>,
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

sos21_domain::delegate_file_repository! {
    impl FileRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_object_repository! {
    impl ObjectRepository for Context {
        Self { S3 },
        self { &self.s3 }
    }
}

sos21_domain::delegate_file_sharing_repository! {
    impl FileSharingRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_file_distribution_repository! {
    impl FileDistributionRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_pending_project_repository! {
    impl PendingProjectRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_registration_form_repository! {
    impl RegistrationFormRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_registration_form_answer_repository! {
    impl RegistrationFormAnswerRepository for Context {
        self { &self.database }
    }
}

sos21_domain::delegate_user_invitation_repository! {
    impl UserInvitationRepository for Context {
        self { &self.database }
    }
}

impl sos21_domain::context::ConfigContext for Context {
    fn administrator_email(&self) -> &UserEmailAddress {
        &self.administrator_email
    }

    fn project_creation_period_for(&self, category: ProjectCategory) -> ProjectCreationPeriod {
        self.project_creation_periods
            .get(&category)
            .copied()
            .unwrap_or_else(ProjectCreationPeriod::never)
    }
}
