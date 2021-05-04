macro_rules! domain_ensure {
    ($cond:expr) => {
        if !$cond {
            return Err($crate::DomainError::Internal(::anyhow::anyhow!(concat!(
                "Condition failed: `",
                stringify!($cond),
                "`"
            ))));
        }
    };
}

pub mod context;
pub mod model;

mod error;
pub use error::{DomainError, DomainResult};

#[cfg(any(feature = "test", test))]
pub mod test;
