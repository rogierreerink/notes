use aes_gcm::{
    AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use sqlx::SqliteExecutor;
use uuid::Uuid;

use crate::{db, services};

#[derive(Debug, PartialEq)]
pub struct DecryptedNote {
    id: Uuid,
    markdown: String,
}

impl DecryptedNote {
    pub fn new(id: Uuid, markdown: String) -> Self {
        Self { id, markdown }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn set_markdown(&mut self, markdown: String) {
        self.markdown = markdown;
    }

    pub fn encrypt(&self, note_key: &Key<Aes256Gcm>) -> services::Result<EncryptedNote> {
        let markdown_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let markdown_ciphertext = Aes256Gcm::new(note_key)
            .encrypt(&markdown_nonce, self.markdown.as_bytes())
            .map_err(|_| services::Error::EncryptionFailed)?;

        Ok(EncryptedNote {
            id: self.id,
            encrypted_markdown: markdown_ciphertext,
            nonce: markdown_nonce.to_vec(),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct EncryptedNote {
    id: Uuid,
    encrypted_markdown: Vec<u8>,
    nonce: Vec<u8>,
}

impl EncryptedNote {
    pub fn decrypt(&self, note_key: &Key<Aes256Gcm>) -> services::Result<DecryptedNote> {
        let markdown_nonce = Nonce::from_slice(&self.nonce);
        let markdown_buf = Aes256Gcm::new(note_key)
            .decrypt(
                &markdown_nonce,
                aes_gcm::aead::Payload {
                    msg: &self.encrypted_markdown,
                    aad: &[],
                },
            )
            .map_err(|_| services::Error::DecryptionFailed)?;

        Ok(DecryptedNote {
            id: self.id,
            markdown: String::from_utf8(markdown_buf)?,
        })
    }
}

pub async fn store<'e, E>(executor: E, note: EncryptedNote) -> services::Result<()>
where
    E: SqliteExecutor<'e>,
{
    // Store note in database
    db::notes::create(
        executor,
        &db::notes::NoteRow {
            id: note.id,
            encrypted_markdown: note.encrypted_markdown,
            nonce: note.nonce,
        },
    )
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, note_id: &Uuid) -> services::Result<EncryptedNote>
where
    E: SqliteExecutor<'e>,
{
    // Get note from database
    let note = db::notes::get_by_id(executor, note_id).await?;

    Ok(EncryptedNote {
        id: note.id,
        encrypted_markdown: note.encrypted_markdown,
        nonce: note.nonce,
    })
}

#[cfg(test)]
mod tests {
    use aes_gcm::{Aes256Gcm, KeyInit, aead::OsRng};
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::services::{
        self,
        notes::{DecryptedNote, EncryptedNote},
    };

    #[tokio::test]
    async fn store_and_get() {
        let pool = init_db().await;

        let encrypted_note = EncryptedNote {
            id: Uuid::new_v4(),
            encrypted_markdown: vec![0, 1, 2, 3],
            nonce: vec![3, 2, 1, 0],
        };

        services::notes::store(&pool, encrypted_note.clone())
            .await
            .expect("failed to store note");

        assert_eq!(
            services::notes::get_by_id(&pool, &encrypted_note.id)
                .await
                .expect("failed to get note by id"),
            encrypted_note,
        );
    }

    #[tokio::test]
    async fn encrypt_decrypt() {
        let note = DecryptedNote::new(Uuid::new_v4(), "hello, world".to_string());
        let note_key = Aes256Gcm::generate_key(&mut OsRng);
        let encrypted_note = note.encrypt(&note_key).expect("failed to encrypt note");

        assert_eq!(
            encrypted_note
                .decrypt(&note_key)
                .expect("failed to decrypt note"),
            note
        );
    }
}
