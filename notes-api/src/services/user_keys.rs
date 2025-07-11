use aes_gcm::{
    AeadCore, Aes256Gcm, Key, KeyInit,
    aead::{Aead, OsRng},
};
use argon2::{Argon2, PasswordHasher};
use password_hash::SaltString;
use sqlx::SqliteExecutor;
use uuid::Uuid;

use crate::db;

#[derive(Debug, PartialEq)]
pub struct UserKey {
    id: Uuid,
    key: Key<Aes256Gcm>,
}

impl UserKey {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            key: Aes256Gcm::generate_key(&mut OsRng),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}

pub async fn store_using_password<'e, E>(
    executor: E,
    user_id: &Uuid,
    user_key: &UserKey,
    password: &str,
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    // Hash the password for use as the encryption key
    let password_salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &password_salt)
        .map_err(|e| anyhow::anyhow!("failed to hash password: {}", e))?
        .hash
        .ok_or(anyhow::anyhow!("empty password hash"))?
        .as_bytes()[0..32]
        .to_vec();

    // Encrypt the user key
    let user_key_key = Key::<Aes256Gcm>::from_slice(&password_hash);
    let user_key_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let user_key_ciphertext = Aes256Gcm::new(user_key_key)
        .encrypt(
            &user_key_nonce,
            aes_gcm::aead::Payload {
                msg: &user_key.key,
                aad: &[],
            },
        )
        .map_err(|e| anyhow::anyhow!("failed to encrypt user key: {}", e))?;

    // Convert the password salt to a byte array
    let mut password_salt_buf = [0u8; 16];
    password_salt
        .decode_b64(&mut password_salt_buf)
        .map_err(|e| anyhow::anyhow!("failed to decode password salt: {}", e))?;

    // Store the encrypted user key, nonce and password salt
    db::user_keys::create(
        executor,
        &db::user_keys::UserKeyRow {
            id: user_key.id,
            user_id: *user_id,
            encrypted_key: user_key_ciphertext,
            nonce: user_key_nonce.to_vec(),
            salt: password_salt_buf.to_vec(),
        },
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::{db, services};

    #[tokio::test]
    async fn store_using_password() {
        let pool = init_db().await;

        // Populate database

        let user_id = Uuid::new_v4();

        db::users::create(
            &pool,
            &db::users::UserRow {
                id: user_id,
                username: "test".to_string(),
            },
        )
        .await
        .expect("failed to create user");

        // Perform test

        let password = "1234";
        let user_key = services::user_keys::UserKey::new();

        services::user_keys::store_using_password(&pool, &user_id, &user_key, password)
            .await
            .expect("failed to store key using password");
    }
}
