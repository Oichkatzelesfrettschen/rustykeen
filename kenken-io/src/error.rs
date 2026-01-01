use kenken_core::CoreError;

#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[cfg(feature = "io-rkyv")]
    #[error(transparent)]
    Rkyv(#[from] rkyv::rancor::Error),

    #[error("invalid snapshot magic")]
    InvalidSnapshotMagic,

    #[error("invalid snapshot data")]
    InvalidSnapshotData,
}
