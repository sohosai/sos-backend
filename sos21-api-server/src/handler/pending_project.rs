pub mod registration_form;

pub mod get;
pub use get::handler as get;
pub mod update;
pub use update::handler as update;
pub mod update_any;
pub use update_any::handler as update_any;
