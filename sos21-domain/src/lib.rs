pub mod context;
pub mod model;

mod error;
pub use error::{DomainError, DomainResult};

#[cfg(any(feature = "test", test))]
pub mod test;
