use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug, PartialEq)]
pub struct NoteKey {
    pub id: Uuid,
    pub note_id: Uuid,
    pub user_id: Uuid,
    pub encrypted_key: String,
}

pub async fn create<'e, E>(executor: E, note_key: &NoteKey) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO note_keys (id, note_id, user_id, encrypted_key)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(&note_key.id)
    .bind(&note_key.note_id)
    .bind(&note_key.user_id)
    .bind(&note_key.encrypted_key)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> anyhow::Result<NoteKey>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, note_id, user_id, encrypted_key
        FROM note_keys
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(executor)
    .await?)
}

#[cfg(test)]
mod tests {
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::db::note_keys::{self, NoteKey};

    #[tokio::test]
    async fn create() {
        let pool = init_db().await;

        // Populate database

        let note_id = Uuid::new_v4();
        let encryted_markdown = "1234".to_string();

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown)
            VALUES (?1, ?2)
            "#,
        )
        .bind(&note_id)
        .bind(&encryted_markdown)
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

        let note_key = NoteKey {
            id: Uuid::new_v4(),
            note_id,
            user_id,
            encrypted_key: "5678".to_string(),
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
        let encryted_markdown = "1234".to_string();

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown)
            VALUES (?1, ?2)
            "#,
        )
        .bind(&note_id)
        .bind(&encryted_markdown)
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
        let encrypted_key = "5678".to_string();

        sqlx::query(
            r#"
            INSERT INTO note_keys (id, note_id, user_id, encrypted_key)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(&id)
        .bind(&note_id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .execute(&pool)
        .await
        .expect("failed to insert note_key");

        // Perform test

        assert_eq!(
            note_keys::get_by_id(&pool, &id)
                .await
                .expect("failed to get note key by id"),
            NoteKey {
                id,
                note_id,
                user_id,
                encrypted_key
            }
        )
    }
}
