#[derive(Debug)]
pub enum DomainError<E> {
    Domain(E),
    Internal(anyhow::Error),
}

impl<E> From<anyhow::Error> for DomainError<E> {
    fn from(e: anyhow::Error) -> Self {
        DomainError::Internal(e)
    }
}

pub type DomainResult<T, E> = Result<T, DomainError<E>>;
