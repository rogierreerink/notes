use argon2::{Argon2, PasswordHasher};
use password_hash::SaltString;
use sqlx::SqliteExecutor;
use uuid::Uuid;

use crate::db;

#[derive(Debug, PartialEq)]
pub struct UserPassword {
    hash: Vec<u8>,
    salt: SaltString,
}

pub async fn get<'e, E>(executor: E, user_id: &Uuid) -> anyhow::Result<UserPassword>
where
    E: SqliteExecutor<'e>,
{
    // Get the user password from the database
    Ok(db::user_passwords::get_by_user_id(executor, user_id)
        .await
        .map(|user_password| UserPassword {
            hash: user_password.hash,
            salt: SaltString::encode_b64(&user_password.salt)
                .expect("failed to encode salt string"),
        })?)
}

pub async fn validate(
    user_password: &UserPassword,
    provided_password: &str,
) -> anyhow::Result<bool> {
    // Recreate the user password hash from `user_password` and user password salt
    let provided_hash = Argon2::default()
        .hash_password(provided_password.as_bytes(), &user_password.salt)
        .expect("failed to hash user password")
        .hash
        .expect("failed to get user password hash")
        .as_bytes()[..32]
        .to_vec();

    // Check that the hashes match
    Ok(user_password.hash == provided_hash)
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
    async fn get_existing_user_password() {
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
            &db::user_passwords::UserPasswordRow {
                id: user_password_id,
                user_id,
                user_key_id,
                hash: user_password_hash.clone(),
                salt: user_password_salt_buf,
            },
        )
        .await
        .expect("failed to create user password");

        // Perform test

        assert_eq!(
            services::user_passwords::get(&pool, &user_id)
                .await
                .expect("failed to get user password"),
            services::user_passwords::UserPassword {
                hash: user_password_hash,
                salt: user_password_salt
            }
        );
    }

    #[tokio::test]
    #[ignore]
    async fn get_nonexisting_user_password() {
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

        todo!("implement Error type")
        // services::user_passwords::get(&pool, &user_id)
        //     .await
        //     .expect("failed to get user password");
    }

    #[tokio::test]
    async fn validate_valid_password() {
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
            &db::user_passwords::UserPasswordRow {
                id: user_password_id,
                user_id,
                user_key_id,
                hash: user_password_hash,
                salt: user_password_salt_buf,
            },
        )
        .await
        .expect("failed to create user password");

        // Perform test

        let provided_password = user_password;
        let user_password = services::user_passwords::get(&pool, &user_id)
            .await
            .expect("failed to get user password");

        assert!(
            services::user_passwords::validate(&user_password, &provided_password)
                .await
                .expect("failed to validate password")
        );
    }

    #[tokio::test]
    async fn validate_invalid_password() {
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
            &db::user_passwords::UserPasswordRow {
                id: user_password_id,
                user_id,
                user_key_id,
                hash: user_password_hash,
                salt: user_password_salt_buf,
            },
        )
        .await
        .expect("failed to create user password");

        // Perform test

        let provided_password = "4321".to_string();
        let user_password = services::user_passwords::get(&pool, &user_id)
            .await
            .expect("failed to get user password");

        assert!(
            !services::user_passwords::validate(&user_password, &provided_password)
                .await
                .expect("failed to validate password")
        );
    }
}
