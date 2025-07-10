use aes_gcm::{Aes256Gcm, Key};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod notes;

#[derive(Debug)]
pub struct Note {
    pub id: Uuid,
    pub markdown: String,
}

#[derive(Debug)]
pub struct NoteKey {
    pub id: Uuid,
    pub user_key: UserKey,
    pub key: Key<Aes256Gcm>,
}

pub mod user_passwords;
pub mod user_sessions;
pub mod users;

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug)]
pub struct UserKey {
    pub id: Uuid,
    pub key: Key<Aes256Gcm>,
}

#[derive(Debug)]
pub struct UserPassword {
    pub id: Uuid,
    pub password: String,
}

#[derive(Debug)]
pub struct UserSession {
    pub id: Uuid,
    pub expiration_time: DateTime<Utc>,
}
