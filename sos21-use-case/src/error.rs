pub enum UseCaseError<E> {
    UseCase(E),
    Internal(anyhow::Error),
}

impl<E> From<anyhow::Error> for UseCaseError<E> {
    fn from(e: anyhow::Error) -> Self {
        UseCaseError::Internal(e)
    }
}

pub type UseCaseResult<T, E> = Result<T, UseCaseError<E>>;
