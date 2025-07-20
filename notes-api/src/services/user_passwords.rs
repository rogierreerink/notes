use aes_gcm::aead::OsRng;
use argon2::{Argon2, PasswordHasher};
use password_hash::SaltString;
use sqlx::SqliteExecutor;
use uuid::Uuid;

use crate::db;

#[derive(Debug, PartialEq)]
pub struct UserPassword {
    id: Uuid,
    user_key_id: Uuid,
    hash: Vec<u8>,
    salt: SaltString,
}

impl UserPassword {
    pub fn new(user_key_id: &Uuid, password: &str) -> anyhow::Result<Self> {
        // Generate a salt and hash the password
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("failed to hash user password: {}", e))?
            .hash
            .ok_or(anyhow::anyhow!("failed to get user password hash"))?
            .as_bytes()
            .to_vec();

        Ok(Self {
            id: Uuid::new_v4(),
            user_key_id: *user_key_id,
            hash,
            salt,
        })
    }

    pub fn verify(&self, password: &str) -> anyhow::Result<bool> {
        // Recreate the user password hash from `user_password` and user password salt
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &self.salt)
            .map_err(|e| anyhow::anyhow!("failed to hash user password: {}", e))?
            .hash
            .ok_or(anyhow::anyhow!("failed to get user password hash"))?
            .as_bytes()
            .to_vec();

        // Check that the hashes match
        Ok(self.hash == hash)
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn user_key_id(&self) -> &Uuid {
        &self.user_key_id
    }
}

pub async fn store<'e, E>(
    executor: E,
    user_password: &UserPassword,
    user_id: &Uuid,
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    // Decode the salt
    let mut salt_buf = vec![0; 16];
    user_password
        .salt
        .decode_b64(&mut salt_buf)
        .map_err(|e| anyhow::anyhow!("failed to decode user password salt: {}", e))?;

    // Store the user password
    db::user_passwords::create(
        executor,
        &db::user_passwords::UserPasswordRow {
            id: user_password.id,
            user_id: *user_id,
            user_key_id: user_password.user_key_id,
            hash: user_password.hash.clone(),
            salt: salt_buf,
        },
    )
    .await?;

    Ok(())
}

pub async fn get_by_user_id<'e, E>(executor: E, user_id: &Uuid) -> anyhow::Result<UserPassword>
where
    E: SqliteExecutor<'e>,
{
    // Get the user password
    let user_password_row = db::user_passwords::get_by_user_id(executor, user_id).await?;

    Ok(UserPassword {
        id: user_password_row.id,
        user_key_id: user_password_row.user_key_id,
        hash: user_password_row.hash,
        salt: SaltString::encode_b64(&user_password_row.salt)
            .map_err(|e| anyhow::anyhow!("failed to encode salt string: {}", e))?,
    })
}

#[cfg(test)]
mod tests {
    use aes_gcm::{
        AeadCore, Aes256Gcm, Key, KeyInit,
        aead::{Aead, OsRng},
    };
    use argon2::{Argon2, PasswordHasher};
    use password_hash::SaltString;
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::{db, services};

    #[tokio::test]
    async fn store() {
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

        let password_str = "1234".to_string();

        let user_key_password_salt = SaltString::generate(&mut OsRng);
        let user_key_password_hash = Argon2::default()
            .hash_password(password_str.as_bytes(), &user_key_password_salt)
            .expect("failed to hash user key password")
            .hash
            .expect("failed to get user key password hash")
            .as_bytes()[..32]
            .to_vec();
        let mut user_key_password_salt_buf = vec![0; 16];
        user_key_password_salt
            .decode_b64(&mut user_key_password_salt_buf)
            .expect("failed to decode user key password salt");

        let user_key_id = Uuid::new_v4();
        let user_key = Key::<Aes256Gcm>::from_slice(&user_key_password_hash);
        let user_key_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let user_key_ciphertext = Aes256Gcm::new(&user_key)
            .encrypt(&user_key_nonce, user_key.as_ref())
            .expect("failed to encrypt user key");

        db::user_keys::create(
            &pool,
            &db::user_keys::UserKeyRow {
                id: user_key_id,
                user_id,
                encrypted_key: user_key_ciphertext,
                nonce: user_key_nonce.to_vec(),
                salt: user_key_password_salt_buf,
            },
        )
        .await
        .expect("failed to create user key");

        // Perform test

        let user_password =
            services::user_passwords::UserPassword::new(&user_key_id, &password_str)
                .expect("failed to create user password");

        services::user_passwords::store(&pool, &user_password, &user_id)
            .await
            .expect("failed to store user password");

        let mut user_password_salt_buf = vec![0; 16];
        user_password
            .salt
            .decode_b64(&mut user_password_salt_buf)
            .expect("failed to decode user password salt");

        assert_eq!(
            db::user_passwords::get_by_id(&pool, &user_password.id)
                .await
                .expect("failed to get user password by id"),
            db::user_passwords::UserPasswordRow {
                id: user_password.id,
                user_id,
                user_key_id,
                hash: user_password.hash,
                salt: user_password_salt_buf
            }
        )
    }

    #[tokio::test]
    async fn get_by_user_id() {
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

        let password_str = "1234".to_string();

        let user_key_password_salt = SaltString::generate(&mut OsRng);
        let user_key_password_hash = Argon2::default()
            .hash_password(password_str.as_bytes(), &user_key_password_salt)
            .expect("failed to hash user key password")
            .hash
            .expect("failed to get user key password hash")
            .as_bytes()[..32]
            .to_vec();
        let mut user_key_password_salt_buf = vec![0; 16];
        user_key_password_salt
            .decode_b64(&mut user_key_password_salt_buf)
            .expect("failed to decode user key password salt");

        let user_key_id = Uuid::new_v4();
        let user_key = Key::<Aes256Gcm>::from_slice(&user_key_password_hash);
        let user_key_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let user_key_ciphertext = Aes256Gcm::new(&user_key)
            .encrypt(&user_key_nonce, user_key.as_ref())
            .expect("failed to encrypt user key");

        db::user_keys::create(
            &pool,
            &db::user_keys::UserKeyRow {
                id: user_key_id,
                user_id,
                encrypted_key: user_key_ciphertext,
                nonce: user_key_nonce.to_vec(),
                salt: user_key_password_salt_buf,
            },
        )
        .await
        .expect("failed to create user key");

        // Perform test

        let user_password =
            services::user_passwords::UserPassword::new(&user_key_id, &password_str)
                .expect("failed to create user password");

        services::user_passwords::store(&pool, &user_password, &user_id)
            .await
            .expect("failed to store user password");

        assert_eq!(
            services::user_passwords::get_by_user_id(&pool, &user_id)
                .await
                .expect("failed to get user password"),
            user_password
        );
    }

    #[tokio::test]
    async fn verify_valid_password() {
        let user_key_id = Uuid::new_v4();
        let password_str = "1234";
        let user_password = services::user_passwords::UserPassword::new(&user_key_id, password_str)
            .expect("failed to create user password");

        assert!(
            user_password
                .verify(password_str)
                .expect("failed to verify password")
        );
    }

    #[tokio::test]
    async fn verify_invalid_password() {
        let user_key_id = Uuid::new_v4();
        let user_password = services::user_passwords::UserPassword::new(&user_key_id, "1234")
            .expect("failed to create user password");

        assert!(
            !user_password
                .verify("4321")
                .expect("failed to verify password")
        );
    }
}
