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

pub async fn get_by_user_id<'e, E>(executor: E, user_id: &Uuid) -> db::Result<Vec<NoteKeyRow>>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT
            note_keys.id,
            note_keys.note_id,
            note_keys.user_id,
            note_keys.encrypted_key,
            note_keys.nonce
        FROM note_keys
            LEFT JOIN notes
                ON note_keys.note_id = notes.id
        WHERE user_id = ?1
        ORDER BY notes.time_created DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
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

pub async fn delete_by_note_id_and_user_id<'e, E>(
    executor: E,
    note_id: &Uuid,
    user_id: &Uuid,
) -> db::Result<()>
where
    E: SqliteExecutor<'e>,
{
    match sqlx::query(
        r#"
        DELETE FROM note_keys
        WHERE note_id = ?1 AND user_id = ?2
        "#,
    )
    .bind(note_id)
    .bind(user_id)
    .execute(executor)
    .await?
    .rows_affected()
    {
        x if x < 1 => Err(db::Error::NotFound),
        x if x > 1 => Err(db::Error::TooMany),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::db::{
        self,
        note_keys::{self, NoteKeyRow},
    };

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
    async fn get_by_user_id() {
        let pool = init_db().await;

        // Populate database

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

        let note_id_1 = Uuid::new_v4();
        let encryted_markdown_1 = vec![1, 2, 3, 4];
        let nonce_1 = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown, nonce)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&note_id_1)
        .bind(&encryted_markdown_1)
        .bind(&nonce_1)
        .execute(&pool)
        .await
        .expect("failed to insert note 1");

        let note_id_2 = Uuid::new_v4();
        let encryted_markdown_2 = vec![4, 3, 2, 1];
        let nonce_2 = vec![8, 7, 6, 5];

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown, nonce)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&note_id_2)
        .bind(&encryted_markdown_2)
        .bind(&nonce_2)
        .execute(&pool)
        .await
        .expect("failed to insert note 2");

        let id_1 = Uuid::new_v4();
        let encrypted_key_1 = vec![1, 2, 3, 4];
        let nonce_1 = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO note_keys (id, note_id, user_id, encrypted_key, nonce)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id_1)
        .bind(&note_id_1)
        .bind(&user_id)
        .bind(&encrypted_key_1)
        .bind(&nonce_1)
        .execute(&pool)
        .await
        .expect("failed to insert note key 1");

        let id_2 = Uuid::new_v4();
        let encrypted_key_2 = vec![1, 2, 3, 4];
        let nonce_2 = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO note_keys (id, note_id, user_id, encrypted_key, nonce)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id_2)
        .bind(&note_id_2)
        .bind(&user_id)
        .bind(&encrypted_key_2)
        .bind(&nonce_2)
        .execute(&pool)
        .await
        .expect("failed to insert note key 2");

        // Perform test

        assert_eq!(
            note_keys::get_by_user_id(&pool, &user_id)
                .await
                .expect("failed to get note keys by user id"),
            vec![
                NoteKeyRow {
                    id: id_1,
                    note_id: note_id_1,
                    user_id,
                    encrypted_key: encrypted_key_1,
                    nonce: nonce_1
                },
                NoteKeyRow {
                    id: id_2,
                    note_id: note_id_2,
                    user_id,
                    encrypted_key: encrypted_key_2,
                    nonce: nonce_2
                }
            ]
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

    #[tokio::test]
    async fn delete_by_note_id_and_user_id() {
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

        note_keys::delete_by_note_id_and_user_id(&pool, &note_id, &user_id)
            .await
            .expect("failed to delete note key by note id and user id");

        assert!(
            note_keys::delete_by_note_id_and_user_id(&pool, &note_id, &user_id)
                .await
                .is_err_and(|e| matches!(e, db::Error::NotFound))
        );
    }
}
