use thiserror::Error;

#[derive(Debug, Error)]
pub enum SolveError {
    #[error("not implemented")]
    NotImplemented,

    #[error(transparent)]
    Core(#[from] kenken_core::CoreError),
}
