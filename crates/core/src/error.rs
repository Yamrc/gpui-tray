use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Tray not found")]
    NotFound,

    #[error("Invalid icon data")]
    InvalidIcon,

    #[error("Already initialized")]
    AlreadyInitialized,
}

pub type Result<T> = std::result::Result<T, Error>;
