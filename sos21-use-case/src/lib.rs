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

pub mod answer_form;
pub mod answer_registration_form;
pub mod create_file;
pub mod create_form;
pub mod create_project;
pub mod create_registration_form;
pub mod delete_user_invitation;
pub mod distribute_files;
pub mod export_form_answers;
pub mod export_projects;
pub mod export_registration_form_answers;
pub mod export_users;
pub mod get_distributed_file;
pub mod get_file;
pub mod get_file_distribution;
pub mod get_file_object;
pub mod get_file_sharing;
pub mod get_form;
pub mod get_form_answer;
pub mod get_form_answer_shared_file;
pub mod get_form_answer_shared_file_object;
pub mod get_login_user;
pub mod get_pending_project;
pub mod get_pending_project_registration_form;
pub mod get_pending_project_registration_form_answer;
pub mod get_project;
pub mod get_project_by_code;
pub mod get_project_form;
pub mod get_project_form_answer;
pub mod get_project_form_answer_shared_file;
pub mod get_project_form_answer_shared_file_object;
pub mod get_project_registration_form;
pub mod get_project_registration_form_answer;
pub mod get_project_registration_form_answer_shared_file;
pub mod get_project_registration_form_answer_shared_file_object;
pub mod get_project_shared_file;
pub mod get_project_shared_file_object;
pub mod get_publicly_shared_file;
pub mod get_publicly_shared_file_object;
pub mod get_registration_form;
pub mod get_registration_form_answer;
pub mod get_registration_form_answer_shared_file;
pub mod get_registration_form_answer_shared_file_object;
pub mod get_shared_file;
pub mod get_shared_file_object;
pub mod get_user;
pub mod get_user_file_usage;
pub mod get_user_invitation;
pub mod get_user_pending_project;
pub mod get_user_project;
pub mod invite_user;
pub mod list_all_file_distributions;
pub mod list_all_forms;
pub mod list_all_projects;
pub mod list_all_registration_forms;
pub mod list_all_user_invitations;
pub mod list_distributed_files;
pub mod list_form_answers;
pub mod list_pending_project_registration_forms;
pub mod list_project_forms;
pub mod list_project_registration_forms;
pub mod list_registration_form_answers;
pub mod list_user_file_sharings;
pub mod list_user_files;
pub mod list_users;
pub mod prepare_project;
pub mod revoke_file_sharing;
pub mod share_file;
pub mod signup;
pub mod update_any_project;
pub mod update_any_user;
pub mod update_form;
pub mod update_pending_project;
pub mod update_project;
pub mod update_project_form_answer;

mod error;
pub use error::{UseCaseError, UseCaseResult};

pub mod interface;
pub mod model;

#[cfg(test)]
pub mod test;
