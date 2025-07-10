use aes_gcm::{
    AeadCore, Aes256Gcm, KeyInit,
    aead::{Aead, OsRng},
};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    db,
    services::{Note, User, UserKey, UserSession},
};

pub async fn create_note(
    db: &SqlitePool,
    user: &User,
    user_session: &UserSession,
    user_key: &UserKey,
    note: &Note,
) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;

    let user_session = db::user_sessions::get_by_id(&mut *tx, &user_session.id).await;
    if user_session.is_err()
        || user_session.is_ok_and(|session| {
            session.user_id != user.id || session.expiration_time.is_some_and(|t| t > Utc::now())
        })
    {
        todo!("session expired")
    }

    let note_key = Aes256Gcm::generate_key(&mut OsRng);
    let note_key_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let note_key_ciphertext = Aes256Gcm::new(&user_key.key)
        .encrypt(&note_key_nonce, note_key.as_ref())
        .unwrap();
    db::note_keys::create(
        &mut *tx,
        &db::note_keys::NoteKeyRow {
            id: Uuid::new_v4(),
            note_id: note.id,
            user_id: user.id,
            encrypted_key: note_key_ciphertext,
            nonce: note_key_nonce.to_vec(),
        },
    )
    .await?;

    let markdown_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let markdown_ciphertext = Aes256Gcm::new(&note_key)
        .encrypt(&markdown_nonce, note.markdown.as_bytes())
        .unwrap();
    db::notes::create(
        &mut *tx,
        &db::notes::NoteRow {
            id: note.id,
            encrypted_markdown: markdown_ciphertext,
            nonce: markdown_nonce.to_vec(),
        },
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn create_note() {}
}
