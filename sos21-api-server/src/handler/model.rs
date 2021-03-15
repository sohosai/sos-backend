//! API model types.
//!
//! API's model types are implemented independently from `sos21_use_case::model`.
//! We never use data transfer object from `sos21-use-case` directly as API's model types,
//! because it's horribly not good that changes in `sos21-use-case` DTO affects
//! the public interface, or specification, of the API server.
//! In the other words, we want to keep the public interface of the API server self-contained in `sos21-api-server`.

pub mod date_time;
pub mod file;
pub mod form;
pub mod form_answer;
pub mod project;
pub mod project_query;
pub mod user;
