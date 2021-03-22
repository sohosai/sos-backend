use futures::lock::Mutex;
use ref_cast::RefCast;
use sqlx::{Postgres, Transaction};

mod project_repository;
use project_repository::ProjectDatabase;
mod form_repository;
use form_repository::FormDatabase;
mod file_repository;
use file_repository::FileDatabase;
mod file_sharing_repository;
use file_sharing_repository::FileSharingDatabase;
mod form_answer_repository;
use form_answer_repository::FormAnswerDatabase;
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

sos21_domain::delegate_project_repository! {
    impl ProjectRepository for Database {
        self { ProjectDatabase::ref_cast(&self.connection) }
    }
}

sos21_domain::delegate_user_repository! {
    impl UserRepository for Database {
        self { UserDatabase::ref_cast(&self.connection) }
    }
}

sos21_domain::delegate_form_repository! {
    impl FormRepository for Database {
        self { FormDatabase::ref_cast(&self.connection) }
    }
}

sos21_domain::delegate_form_answer_repository! {
    impl FormAnswerRepository for Database {
        self { FormAnswerDatabase::ref_cast(&self.connection) }
    }
}

sos21_domain::delegate_file_repository! {
    impl FileRepository for Database {
        self { FileDatabase::ref_cast(&self.connection) }
    }
}

sos21_domain::delegate_file_sharing_repository! {
    impl FileSharingRepository for Database {
        self { FileSharingDatabase::ref_cast(&self.connection) }
    }
}
