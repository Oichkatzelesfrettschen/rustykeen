use thiserror::Error;

#[derive(Debug, Error)]
pub enum SolveError {
    #[error("not implemented")]
    NotImplemented,

    #[error("grid size N={n} not supported by this configuration. {hint}")]
    GridSizeTooLarge { n: u8, hint: String },

    #[error(transparent)]
    Core(#[from] kenken_core::CoreError),
}
