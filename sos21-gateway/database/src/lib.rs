use sqlx::postgres::PgPool;

mod project_repository;
mod user_repository;

#[derive(Debug, Clone)]
pub struct Database {
    project_repository: project_repository::Database,
    user_repository: user_repository::Database,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Database {
            project_repository: project_repository::Database::new(pool.clone()),
            user_repository: user_repository::Database::new(pool),
        }
    }
}

sos21_domain_context::delegate_project_repository! { impl<> for Database : project_repository }
sos21_domain_context::delegate_user_repository! { impl<> for Database : user_repository }
