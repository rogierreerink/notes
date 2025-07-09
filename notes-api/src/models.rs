use aes_gcm::{Aes256Gcm, Key};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct User {
    pub id: Option<Uuid>,
    pub username: Option<String>,
    pub key: Option<UserKey>,
    pub passwords: Option<Vec<UserPassword>>,
    pub sessions: Option<Vec<UserSession>>,
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
