use futures::lock::Mutex;
use ref_cast::RefCast;
use sqlx::{Postgres, Transaction};

mod project_repository;
use project_repository::ProjectDatabase;
mod user_repository;
use user_repository::UserDatabase;

#[derive(Debug)]
pub struct Database {
    connection: Mutex<Transaction<'static, Postgres>>,
}

impl Database {
    pub fn new(connection: Transaction<'static, Postgres>) -> Self {
        Database {
            connection: Mutex::new(connection),
        }
    }

    pub fn into_connection(self) -> Transaction<'static, Postgres> {
        self.connection.into_inner()
    }
}

sos21_domain_context::delegate_project_repository! {
    impl ProjectRepository for Database {
        self { ProjectDatabase::ref_cast(&self.connection) }
    }
}

sos21_domain_context::delegate_user_repository! {
    impl UserRepository for Database {
        self { UserDatabase::ref_cast(&self.connection) }
    }
}
