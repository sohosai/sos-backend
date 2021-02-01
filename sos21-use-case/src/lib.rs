macro_rules! use_case_ensure {
    ($cond:expr) => {
        if !$cond {
            return Err($crate::UseCaseError::Internal(::anyhow::anyhow!(concat!(
                "Condition failed: `",
                stringify!($cond),
                "`"
            ))));
        }
    };
}

pub mod check_project_display_id_status;
pub mod create_project;
pub mod export_projects;
pub mod export_users;
pub mod get_login_user;
pub mod get_project;
pub mod get_project_by_display_id;
pub mod get_user;
pub mod list_all_projects;
pub mod list_user_projects;
pub mod list_users;
pub mod signup;
pub mod update_project;
pub mod update_user;

mod error;
pub use error::{UseCaseError, UseCaseResult};

pub mod model;
