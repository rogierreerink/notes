use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug, PartialEq)]
pub struct Note {
    pub id: Uuid,
    pub encrypted_markdown: String,
}

pub async fn create<'e, E>(executor: E, note: &Note) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO notes (id, encrypted_markdown)
        VALUES (?1, ?2)
        "#,
    )
    .bind(&note.id)
    .bind(&note.encrypted_markdown)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> anyhow::Result<Note>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, encrypted_markdown
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

    use crate::db::notes::{self, Note};

    #[tokio::test]
    async fn create() {
        let pool = init_db().await;

        let note = Note {
            id: Uuid::new_v4(),
            encrypted_markdown: "1234".to_string(),
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
        let encrypted_markdown = "1234".to_string();

        sqlx::query(
            r#"
            INSERT INTO notes (id, encrypted_markdown)
            VALUES (?1, ?2)
            "#,
        )
        .bind(&id)
        .bind(&encrypted_markdown)
        .execute(&pool)
        .await
        .expect("failed to insert note");

        // Perform test

        assert_eq!(
            notes::get_by_id(&pool, &id)
                .await
                .expect("failed to get note by id"),
            Note {
                id,
                encrypted_markdown
            }
        )
    }
}
