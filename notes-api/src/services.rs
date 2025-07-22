use crate::db;

pub mod note_keys;
pub mod notes;

pub mod user_keys;
pub mod user_passwords;
pub mod user_sessions;
pub mod users;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("resource could not be found")]
    NotFound,

    #[error("resource encryption failed")]
    EncryptionFailed,

    #[error("resource decryption failed")]
    DecryptionFailed,

    #[error("internal error: {0}")]
    Internal(anyhow::Error),
}

impl From<db::Error> for Error {
    fn from(e: db::Error) -> Self {
        match e {
            db::Error::NotFound => Self::NotFound,
            db::Error::Internal(_) => Self::Internal(e.into()),
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Internal(e.into())
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}
