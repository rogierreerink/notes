use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug, PartialEq)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
}

pub async fn create<'e, E>(executor: E, user: &UserRow) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO users (id, username)
        VALUES (?1, ?2)
        "#,
    )
    .bind(&user.id)
    .bind(&user.username)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> anyhow::Result<UserRow>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, username
        FROM users
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

    use crate::db::users::{self, UserRow};

    #[tokio::test]
    async fn create() {
        let pool = init_db().await;

        let user = UserRow {
            id: Uuid::new_v4(),
            username: "test".to_string(),
        };

        users::create(&pool, &user)
            .await
            .expect("failed to create user");

        assert_eq!(
            users::get_by_id(&pool, &user.id)
                .await
                .expect("failed to get user"),
            user
        )
    }

    #[tokio::test]
    async fn get_by_id() {
        let pool = init_db().await;

        // Populate database

        let id = Uuid::new_v4();
        let username = "test".to_string();

        sqlx::query(
            r#"
            INSERT INTO users (id, username)
            VALUES (?1, ?2)
            "#,
        )
        .bind(&id)
        .bind(&username)
        .execute(&pool)
        .await
        .expect("failed to insert user");

        // Perform test

        assert_eq!(
            users::get_by_id(&pool, &id)
                .await
                .expect("failed to get user by id"),
            UserRow { id, username }
        )
    }
}
