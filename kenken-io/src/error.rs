use kenken_core::CoreError;

#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[cfg(feature = "io-rkyv")]
    #[error(transparent)]
    Rkyv(#[from] rkyv::rancor::Error),

    #[cfg(feature = "format-sgt-desc")]
    #[error(transparent)]
    SgtDesc(#[from] kenken_core::format::sgt_desc::SgtDescError),

    #[error("invalid snapshot magic")]
    InvalidSnapshotMagic,

    #[error("invalid snapshot data")]
    InvalidSnapshotData,
}
