use sos21_domain::DomainError;

#[derive(Debug)]
pub enum UseCaseError<E> {
    UseCase(E),
    Internal(anyhow::Error),
}

impl<T> UseCaseError<T> {
    pub fn map_use_case<U, F>(self, op: F) -> UseCaseError<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            UseCaseError::UseCase(err) => UseCaseError::UseCase(op(err)),
            UseCaseError::Internal(err) => UseCaseError::Internal(err),
        }
    }

    pub fn from_domain<U, F>(err: DomainError<U>, op: F) -> Self
    where
        F: FnOnce(U) -> T,
    {
        match err {
            DomainError::Domain(err) => UseCaseError::UseCase(op(err)),
            DomainError::Internal(err) => UseCaseError::Internal(err),
        }
    }
}

impl<E> From<anyhow::Error> for UseCaseError<E> {
    fn from(e: anyhow::Error) -> Self {
        UseCaseError::Internal(e)
    }
}

pub type UseCaseResult<T, E> = Result<T, UseCaseError<E>>;
