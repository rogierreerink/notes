use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

use crate::db;

#[derive(FromRow, Debug, PartialEq)]
pub struct NoteKeyRow {
    pub id: Uuid,
    pub note_id: Uuid,
    pub user_id: Uuid,
    pub encrypted_key: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub async fn create<'e, E>(executor: E, note_key: &NoteKeyRow) -> db::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO note_keys (id, note_id, user_id, encrypted_key, nonce)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(&note_key.id)
    .bind(&note_key.note_id)
    .bind(&note_key.user_id)
    .bind(&note_key.encrypted_key)
    .bind(&note_key.nonce)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> db::Result<NoteKeyRow>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, note_id, user_id, encrypted_key, nonce
        FROM note_keys
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(executor)
    .await?)
}

pub async fn get_by_note_id_and_user_id<'e, E>(
    executor: E,
    note_id: &Uuid,
    user_id: &Uuid,
) -> db::Result<NoteKeyRow>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, note_id, user_id, encrypted_key, nonce
        FROM note_keys
        WHERE note_id = ?1 AND user_id = ?2
        "#,
    )
    .bind(note_id)
    .bind(user_id)
    .fetch_one(executor)
    .await?)
}

#[cfg(test)]
mod tests {
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::db::note_keys::{self, NoteKeyRow};

    #[tokio::test]
    async fn create() {
        let pool = init_db().await;

        // Populate database

        let note_id = Uuid::new_v4();
        let encryted_markdown = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown, nonce)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&note_id)
        .bind(&encryted_markdown)
        .bind(&nonce)
        .execute(&pool)
        .await
        .expect("failed to insert note");

        let user_id = Uuid::new_v4();
        let username = "test".to_string();

        sqlx::query(
            r#"
            INSERT INTO users (id, username)
            VALUES (?1, ?2)
            "#,
        )
        .bind(&user_id)
        .bind(&username)
        .execute(&pool)
        .await
        .expect("failed to insert user");

        // Perform test

        let note_key = NoteKeyRow {
            id: Uuid::new_v4(),
            note_id,
            user_id,
            encrypted_key: vec![1, 2, 3, 4],
            nonce: vec![5, 6, 7, 8],
        };

        note_keys::create(&pool, &note_key)
            .await
            .expect("failed to create note key");

        assert_eq!(
            note_keys::get_by_id(&pool, &note_key.id)
                .await
                .expect("failed to get note key"),
            note_key
        )
    }

    #[tokio::test]
    async fn get_by_id() {
        let pool = init_db().await;

        // Populate database

        let note_id = Uuid::new_v4();
        let encryted_markdown = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown, nonce)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&note_id)
        .bind(&encryted_markdown)
        .bind(&nonce)
        .execute(&pool)
        .await
        .expect("failed to insert note");

        let user_id = Uuid::new_v4();
        let username = "test".to_string();

        sqlx::query(
            r#"
            INSERT INTO users (id, username)
            VALUES (?1, ?2)
            "#,
        )
        .bind(&user_id)
        .bind(&username)
        .execute(&pool)
        .await
        .expect("failed to insert user");

        let id = Uuid::new_v4();
        let encrypted_key = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO note_keys (id, note_id, user_id, encrypted_key, nonce)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id)
        .bind(&note_id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .execute(&pool)
        .await
        .expect("failed to insert note_key");

        // Perform test

        assert_eq!(
            note_keys::get_by_id(&pool, &id)
                .await
                .expect("failed to get note key by id"),
            NoteKeyRow {
                id,
                note_id,
                user_id,
                encrypted_key,
                nonce
            }
        )
    }

    #[tokio::test]
    async fn get_by_note_id_and_user_id() {
        let pool = init_db().await;

        // Populate database

        let note_id = Uuid::new_v4();
        let encryted_markdown = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown, nonce)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&note_id)
        .bind(&encryted_markdown)
        .bind(&nonce)
        .execute(&pool)
        .await
        .expect("failed to insert note");

        let user_id = Uuid::new_v4();
        let username = "test".to_string();

        sqlx::query(
            r#"
            INSERT INTO users (id, username)
            VALUES (?1, ?2)
            "#,
        )
        .bind(&user_id)
        .bind(&username)
        .execute(&pool)
        .await
        .expect("failed to insert user");

        let id = Uuid::new_v4();
        let encrypted_key = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO note_keys (id, note_id, user_id, encrypted_key, nonce)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id)
        .bind(&note_id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .execute(&pool)
        .await
        .expect("failed to insert note_key");

        // Perform test

        assert_eq!(
            note_keys::get_by_note_id_and_user_id(&pool, &note_id, &user_id)
                .await
                .expect("failed to get note key by note id and user id"),
            NoteKeyRow {
                id,
                note_id,
                user_id,
                encrypted_key,
                nonce
            }
        )
    }
}
