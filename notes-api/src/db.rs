pub mod user_keys;
pub mod user_passwords;
pub mod user_sessions;
pub mod users;

pub mod note_keys;
pub mod notes;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("item could not be found")]
    NotFound,

    #[error("too many items matched the query")]
    TooMany,

    #[error("internal error: {0}")]
    Internal(anyhow::Error),
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Internal(e.into()),
        }
    }
}
