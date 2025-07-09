use aes_gcm::{
    Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use argon2::{Argon2, PasswordHasher};
use chrono::{DateTime, Utc};
use password_hash::SaltString;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db;

pub struct UserSession<'a> {
    db: &'a SqlitePool,
    session_id: Uuid,
    user_id: Uuid,
    user_key: Key<Aes256Gcm>,
    expiration_time: Option<DateTime<Utc>>,
}

pub enum UserAuthentication {
    Password(String),
}

impl<'a> UserSession<'a> {
    pub async fn new(
        db: &'a SqlitePool,
        user_id: &Uuid,
        user_auth: &Option<UserAuthentication>,
        expiration_time: &Option<DateTime<Utc>>,
    ) -> anyhow::Result<Self> {
        let mut tx = db.begin().await?;

        // Get or generate the user key
        let user_key = match user_auth {
            // Get the password encrypted user key
            Some(UserAuthentication::Password(password)) => {
                // Get the user password entry from the database
                let user_password = db::user_passwords::get_by_user_id(&mut *tx, user_id).await?;

                // Recreate the user password hash from `password` and user password salt
                let user_password_salt = SaltString::encode_b64(&user_password.salt).unwrap();
                let user_password_hash = Argon2::default()
                    .hash_password(password.as_bytes(), &user_password_salt)
                    .expect("failed to hash user password")
                    .hash
                    .expect("failed to get user password hash")
                    .as_bytes()[..32]
                    .to_vec();

                // Check that the stored hash matches the recreated hash
                if user_password.password_hash != user_password_hash {
                    todo!("incorrect password")
                }

                // Get the user key from the database
                let user_key =
                    db::user_keys::get_by_id(&mut *tx, &user_password.user_key_id).await?;

                // Recreate the encryption key from the user password and user key salt
                let user_key_password_salt = SaltString::encode_b64(&user_key.salt).unwrap();
                let user_key_password_hash = Argon2::default()
                    .hash_password(password.as_bytes(), &user_key_password_salt)
                    .expect("failed to hash user key password")
                    .hash
                    .expect("failed to get user password key hash")
                    .as_bytes()[..32]
                    .to_vec();

                // Decrypt the user key
                let user_key_key = Key::<Aes256Gcm>::from_slice(&user_key_password_hash);
                let user_key_nonce = Nonce::from_slice(&user_key.nonce);
                *Key::<Aes256Gcm>::from_slice(
                    &Aes256Gcm::new(&user_key_key)
                        .decrypt(
                            user_key_nonce,
                            aes_gcm::aead::Payload {
                                msg: &user_key.encrypted_key,
                                aad: &vec![],
                            },
                        )
                        .expect("user key decryption failed"),
                )
            }

            // Generate a new user key
            None => {
                // Check that the user exists
                db::users::get_by_id(&mut *tx, user_id).await?;

                // Check that the user doesn't already own a user key
                if !db::user_keys::get_by_user_id(&mut *tx, user_id)
                    .await?
                    .is_empty()
                {
                    todo!("user already owns a user key")
                };

                // Generate a user key
                Aes256Gcm::generate_key(OsRng)
            }
        };

        // Create a user session
        let session_id = Uuid::new_v4();
        db::user_sessions::create(
            &mut *tx,
            &db::user_sessions::UserSession {
                id: session_id,
                user_id: *user_id,
                expiration_time: *expiration_time,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(Self {
            db,
            session_id,
            user_id: *user_id,
            user_key,
            expiration_time: *expiration_time,
        })
    }

    pub async fn from_jwt(db: &SqlitePool) -> Self {
        todo!()
    }

    pub async fn to_jwt(&self) -> () {
        todo!()
    }

    pub async fn expiration_time(&self) -> &Option<DateTime<Utc>> {
        &self.expiration_time
    }

    pub async fn user_id(&self) -> &Uuid {
        &self.user_id
    }

    pub async fn user_key(&self) -> &Key<Aes256Gcm> {
        &self.user_key
    }
}

#[cfg(test)]
mod tests {
    use aes_gcm::{
        AeadCore, Aes256Gcm, Key, KeyInit,
        aead::{Aead, OsRng},
    };
    use argon2::PasswordHasher;
    use argon2::{Argon2, password_hash::SaltString};
    use chrono::{Duration, Utc};
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::{
        db::{self},
        services::user_sessions::{UserAuthentication, UserSession},
    };

    #[tokio::test]
    async fn create_no_auth() {
        let pool = init_db().await;

        let user_id = Uuid::new_v4();
        let username = "test".to_string();

        db::users::create(
            &pool,
            &db::users::User {
                id: user_id,
                username,
            },
        )
        .await
        .expect("failed to create user");

        let expiration_time = Some(Utc::now() + Duration::minutes(30));
        let user_session = UserSession::new(&pool, &user_id, &None, &expiration_time)
            .await
            .expect("failed to create user session");

        assert_eq!(user_session.user_id, user_id);
        assert_eq!(user_session.expiration_time, expiration_time);
    }

    #[tokio::test]
    async fn create_password_auth() {
        let pool = init_db().await;

        let user_id = Uuid::new_v4();
        let username = "test".to_string();

        db::users::create(
            &pool,
            &db::users::User {
                id: user_id,
                username,
            },
        )
        .await
        .expect("failed to create user");

        let user_password = "1234".to_string();

        let user_key_password_salt = SaltString::generate(&mut OsRng);
        let user_key_password_hash = Argon2::default()
            .hash_password(user_password.as_bytes(), &user_key_password_salt)
            .expect("failed to hash user password")
            .hash
            .expect("failed to get user password hash")
            .as_bytes()[..32]
            .to_vec();
        let mut user_key_password_salt_buf = vec![0; 16];
        user_key_password_salt
            .decode_b64(&mut user_key_password_salt_buf)
            .expect("failed to decode user password salt");

        let user_key_id = Uuid::new_v4();
        let user_key = Key::<Aes256Gcm>::from_slice(&user_key_password_hash);
        let user_key_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let user_key_ciphertext = Aes256Gcm::new(&user_key)
            .encrypt(&user_key_nonce, user_key.as_ref())
            .expect("failed to encrypt user key");

        db::user_keys::create(
            &pool,
            &db::user_keys::UserKey {
                id: user_key_id,
                user_id,
                encrypted_key: user_key_ciphertext,
                nonce: user_key_nonce.to_vec(),
                salt: user_key_password_salt_buf,
            },
        )
        .await
        .expect("failed to create user key");

        let user_password_id = Uuid::new_v4();
        let user_password_salt = SaltString::generate(&mut OsRng);
        let user_password_hash = Argon2::default()
            .hash_password(user_password.as_bytes(), &user_password_salt)
            .expect("failed to hash user password")
            .hash
            .expect("failed to get user password hash")
            .as_bytes()[..32]
            .to_vec();
        let mut user_password_salt_buf = vec![0; 16];
        user_password_salt
            .decode_b64(&mut user_password_salt_buf)
            .expect("failed to decode user password salt");

        db::user_passwords::create(
            &pool,
            &db::user_passwords::UserPassword {
                id: user_password_id,
                user_id,
                user_key_id,
                password_hash: user_password_hash,
                salt: user_password_salt_buf,
            },
        )
        .await
        .expect("failed to create user password");

        let expiration_time = Some(Utc::now() + Duration::minutes(30));
        let user_session = UserSession::new(
            &pool,
            &user_id,
            &Some(UserAuthentication::Password(user_password)),
            &expiration_time,
        )
        .await
        .expect("failed to create user session");

        assert_eq!(user_session.user_id, user_id);
        assert_eq!(user_session.expiration_time, expiration_time);
    }
}
