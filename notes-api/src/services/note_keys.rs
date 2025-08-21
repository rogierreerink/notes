use aes_gcm::{
    AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use sqlx::SqliteExecutor;
use uuid::Uuid;

use crate::{db, services};

#[derive(Debug, PartialEq)]
pub struct DecryptedNoteKey {
    id: Uuid,
    key: Key<Aes256Gcm>,
}

impl DecryptedNoteKey {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            key: Aes256Gcm::generate_key(&mut OsRng),
        }
    }

    pub fn key(&self) -> &Key<Aes256Gcm> {
        &self.key
    }

    pub fn encrypt(&self, user_key: &Key<Aes256Gcm>) -> services::Result<EncryptedNoteKey> {
        let note_key_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let note_key_ciphertext = Aes256Gcm::new(user_key)
            .encrypt(&note_key_nonce, self.key.as_ref())
            .map_err(|_| services::Error::EncryptionFailed)?;

        Ok(EncryptedNoteKey {
            id: self.id,
            encrypted_key: note_key_ciphertext,
            nonce: note_key_nonce.to_vec(),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EncryptedNoteKey {
    id: Uuid,
    encrypted_key: Vec<u8>,
    nonce: Vec<u8>,
}

impl EncryptedNoteKey {
    pub fn decrypt(&self, user_key: &Key<Aes256Gcm>) -> services::Result<DecryptedNoteKey> {
        let note_key_nonce = Nonce::from_slice(&self.nonce);
        let note_key_buf = Aes256Gcm::new(user_key)
            .decrypt(&note_key_nonce, self.encrypted_key.as_ref())
            .map_err(|_| services::Error::DecryptionFailed)?;

        Ok(DecryptedNoteKey {
            id: self.id,
            key: *Key::<Aes256Gcm>::from_slice(&note_key_buf),
        })
    }
}

pub async fn store<'e, E>(
    executor: E,
    note_key: EncryptedNoteKey,
    note_id: &Uuid,
    user_id: &Uuid,
) -> services::Result<()>
where
    E: SqliteExecutor<'e>,
{
    // Store note key in database
    db::note_keys::create(
        executor,
        &db::note_keys::NoteKeyRow {
            id: note_key.id,
            note_id: *note_id,
            user_id: *user_id,
            encrypted_key: note_key.encrypted_key,
            nonce: note_key.nonce,
        },
    )
    .await?;

    Ok(())
}

pub struct NoteKeyLink {
    pub note_id: Uuid,
    pub note_key: EncryptedNoteKey,
}

pub async fn search<'e, E>(executor: E, user_id: &Uuid) -> services::Result<Vec<NoteKeyLink>>
where
    E: SqliteExecutor<'e>,
{
    // Get note keys from database
    let note_key_rows = db::note_keys::get_by_user_id(executor, user_id).await?;

    Ok(note_key_rows
        .iter()
        .map(|row| NoteKeyLink {
            note_id: row.note_id,
            note_key: EncryptedNoteKey {
                id: row.id,
                encrypted_key: row.encrypted_key.clone(),
                nonce: row.nonce.clone(),
            },
        })
        .collect())
}

pub async fn get<'e, E>(
    executor: E,
    note_id: &Uuid,
    user_id: &Uuid,
) -> services::Result<EncryptedNoteKey>
where
    E: SqliteExecutor<'e>,
{
    // Get note key from database
    let note_key_row =
        db::note_keys::get_by_note_id_and_user_id(executor, note_id, user_id).await?;

    Ok(EncryptedNoteKey {
        id: note_key_row.id,
        encrypted_key: note_key_row.encrypted_key,
        nonce: note_key_row.nonce,
    })
}

pub async fn delete<'e, E>(executor: E, note_id: &Uuid, user_id: &Uuid) -> services::Result<()>
where
    E: SqliteExecutor<'e>,
{
    // Delete note key from database
    db::note_keys::delete_by_note_id_and_user_id(executor, note_id, user_id).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use aes_gcm::{Aes256Gcm, KeyInit, aead::OsRng};
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::{
        db,
        services::{
            self,
            note_keys::{DecryptedNoteKey, EncryptedNoteKey},
        },
    };

    #[tokio::test]
    async fn store_and_get() {
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

        let note_id = Uuid::new_v4();

        db::notes::upsert(
            &pool,
            &db::notes::NoteRow {
                id: note_id,
                encrypted_markdown: vec![1, 2, 3, 4],
                nonce: vec![1, 2, 3, 4],
                time_created: None,
            },
        )
        .await
        .expect("failed to create note");

        // Perform test

        let note_key = EncryptedNoteKey {
            id: Uuid::new_v4(),
            encrypted_key: vec![0, 1, 2, 3],
            nonce: vec![3, 2, 1, 0],
        };

        services::note_keys::store(&pool, note_key.clone(), &note_id, &user_id)
            .await
            .expect("failed to store note key");

        assert_eq!(
            services::note_keys::get(&pool, &note_id, &user_id)
                .await
                .expect("failed to get note key"),
            note_key
        )
    }

    #[tokio::test]
    async fn encrypt_decrypt() {
        let user_key = Aes256Gcm::generate_key(&mut OsRng);
        let note_key = DecryptedNoteKey::new();
        let encrypted_note_key = note_key
            .encrypt(&user_key)
            .expect("failed to encrypt note key");

        assert_eq!(
            encrypted_note_key
                .decrypt(&user_key)
                .expect("failed to decrypt note key"),
            note_key
        );
    }
}
