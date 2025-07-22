use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

use crate::db;

#[derive(FromRow, Debug, PartialEq)]
pub struct NoteRow {
    pub id: Uuid,
    pub encrypted_markdown: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub async fn create<'e, E>(executor: E, note: &NoteRow) -> db::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO notes (id, encrypted_markdown, nonce)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(&note.id)
    .bind(&note.encrypted_markdown)
    .bind(&note.nonce)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> db::Result<NoteRow>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, encrypted_markdown, nonce
        FROM notes
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

    use crate::db::notes::{self, NoteRow};

    #[tokio::test]
    async fn create() {
        let pool = init_db().await;

        let note = NoteRow {
            id: Uuid::new_v4(),
            encrypted_markdown: vec![1, 2, 3, 4],
            nonce: vec![5, 6, 7, 8],
        };

        notes::create(&pool, &note)
            .await
            .expect("failed to create note");

        assert_eq!(
            notes::get_by_id(&pool, &note.id)
                .await
                .expect("failed to get note"),
            note
        )
    }

    #[tokio::test]
    async fn get_by_id() {
        let pool = init_db().await;

        // Populate database

        let id = Uuid::new_v4();
        let encrypted_markdown = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown, nonce)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&id)
        .bind(&encrypted_markdown)
        .bind(&nonce)
        .execute(&pool)
        .await
        .expect("failed to insert note");

        // Perform test

        assert_eq!(
            notes::get_by_id(&pool, &id)
                .await
                .expect("failed to get note by id"),
            NoteRow {
                id,
                encrypted_markdown,
                nonce
            }
        )
    }
}
