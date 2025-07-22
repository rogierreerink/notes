use chrono::{DateTime, Utc};
use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

use crate::db;

#[derive(FromRow, Debug, PartialEq)]
pub struct UserSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expiration_time: Option<DateTime<Utc>>,
}

pub async fn create<'e, E>(executor: E, user_session: &UserSessionRow) -> db::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO user_sessions (id, user_id, expiration_time)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(&user_session.id)
    .bind(&user_session.user_id)
    .bind(&user_session.expiration_time)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> db::Result<UserSessionRow>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, user_id, expiration_time
        FROM user_sessions
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(executor)
    .await?)
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::db::user_sessions::{self, UserSessionRow};

    #[tokio::test]
    async fn create() {
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

        // Perform test

        let user_session = UserSessionRow {
            id: Uuid::new_v4(),
            user_id,
            expiration_time: DateTime::from_timestamp(0, 0),
        };

        user_sessions::create(&pool, &user_session)
            .await
            .expect("failed to create user session");

        assert_eq!(
            user_sessions::get_by_id(&pool, &user_session.id)
                .await
                .expect("failed to get user session by id"),
            user_session
        )
    }

    #[tokio::test]
    async fn get_by_id() {
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

        let id = Uuid::new_v4();
        let expiration_time = DateTime::from_timestamp(0, 0);
        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, expiration_time)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&expiration_time)
        .execute(&pool)
        .await
        .expect("failed to insert user session");

        // Perform test

        assert_eq!(
            user_sessions::get_by_id(&pool, &id)
                .await
                .expect("failed to get user session by id"),
            UserSessionRow {
                id,
                user_id,
                expiration_time
            }
        )
    }
}
