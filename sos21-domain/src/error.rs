#[derive(Debug)]
pub enum DomainError<E> {
    Domain(E),
    Internal(anyhow::Error),
}

impl<T> DomainError<T> {
    pub fn map_domain<U, F>(self, op: F) -> DomainError<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            DomainError::Domain(err) => DomainError::Domain(op(err)),
            DomainError::Internal(err) => DomainError::Internal(err),
        }
    }
}

impl<E> From<anyhow::Error> for DomainError<E> {
    fn from(e: anyhow::Error) -> Self {
        DomainError::Internal(e)
    }
}

pub type DomainResult<T, E> = Result<T, DomainError<E>>;
