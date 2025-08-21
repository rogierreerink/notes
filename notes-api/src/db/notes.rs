use chrono::{DateTime, Utc};
use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

use crate::db;

#[derive(FromRow, Debug, PartialEq)]
pub struct NoteRow {
    pub id: Uuid,
    pub encrypted_markdown: Vec<u8>,
    pub nonce: Vec<u8>,

    /// Time created is set by the database server when
    /// when creating a new Note Row. It must therefore
    /// be optional.
    pub time_created: Option<DateTime<Utc>>,
}

pub async fn upsert<'e, E>(executor: E, note: &NoteRow) -> db::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO notes (id, encrypted_markdown, nonce)
        VALUES (?1, ?2, ?3)
        ON CONFLICT (id) DO UPDATE SET
            encrypted_markdown = ?2,
            nonce = ?3
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
        SELECT id, encrypted_markdown, nonce, time_created
        FROM notes
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(executor)
    .await?)
}

pub async fn delete_by_id<'e, E>(executor: E, id: &Uuid) -> db::Result<()>
where
    E: SqliteExecutor<'e>,
{
    match sqlx::query(
        r#"
        DELETE FROM notes
        WHERE id = ?1
        "#,
    )
    .bind(id)
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
        notes::{self, NoteRow},
    };

    #[tokio::test]
    async fn upsert() {
        let pool = init_db().await;

        let mut note = NoteRow {
            id: Uuid::new_v4(),
            encrypted_markdown: vec![1, 2, 3, 4],
            nonce: vec![5, 6, 7, 8],
            time_created: None,
        };

        notes::upsert(&pool, &note)
            .await
            .expect("failed to create note");

        let inserted = notes::get_by_id(&pool, &note.id)
            .await
            .expect("failed to get note");
        assert_eq!(inserted.id, note.id);
        assert_eq!(inserted.encrypted_markdown, note.encrypted_markdown);
        assert_eq!(inserted.nonce, note.nonce);

        note.encrypted_markdown = vec![5, 6, 7, 8];
        note.nonce = vec![1, 2, 3, 4];

        notes::upsert(&pool, &note)
            .await
            .expect("failed to update note");

        let updated = notes::get_by_id(&pool, &note.id)
            .await
            .expect("failed to get note");
        assert_eq!(updated.id, note.id);
        assert_eq!(updated.encrypted_markdown, note.encrypted_markdown);
        assert_eq!(updated.nonce, note.nonce);
    }

    #[tokio::test]
    async fn get_by_id() {
        let pool = init_db().await;

        // Populate database

        let note = NoteRow {
            id: Uuid::new_v4(),
            encrypted_markdown: vec![1, 2, 3, 4],
            nonce: vec![5, 6, 7, 8],
            time_created: None,
        };

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown, nonce)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&note.id)
        .bind(&note.encrypted_markdown)
        .bind(&note.nonce)
        .execute(&pool)
        .await
        .expect("failed to insert note");

        // Perform test

        let inserted = notes::get_by_id(&pool, &note.id)
            .await
            .expect("failed to get note");
        assert_eq!(inserted.id, note.id);
        assert_eq!(inserted.encrypted_markdown, note.encrypted_markdown);
        assert_eq!(inserted.nonce, note.nonce);
    }

    #[tokio::test]
    async fn delete_by_id() {
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

        notes::delete_by_id(&pool, &id)
            .await
            .expect("failed to delete note by id");

        assert!(
            notes::get_by_id(&pool, &id)
                .await
                .is_err_and(|e| matches!(e, db::Error::NotFound))
        )
    }
}
