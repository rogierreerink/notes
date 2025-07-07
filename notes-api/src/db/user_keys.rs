use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug, PartialEq)]
pub struct UserKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub encrypted_key: String,
}

pub async fn create<'e, E>(executor: E, user_key: &UserKey) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO user_keys (id, user_id, encrypted_key)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(&user_key.id)
    .bind(&user_key.user_id)
    .bind(&user_key.encrypted_key)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> anyhow::Result<UserKey>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, user_id, encrypted_key
        FROM user_keys
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

    use crate::db::user_keys::{self, UserKey};

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

        let user_key = UserKey {
            id: Uuid::new_v4(),
            user_id,
            encrypted_key: "1234".to_string(),
        };

        user_keys::create(&pool, &user_key)
            .await
            .expect("failed to create user key");

        assert_eq!(
            user_keys::get_by_id(&pool, &user_key.id)
                .await
                .expect("failed to get user key by id"),
            user_key
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
        let encrypted_key = "1234".to_string();

        sqlx::query(
            r#"
            INSERT INTO user_keys (id, user_id, encrypted_key)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .execute(&pool)
        .await
        .expect("failed to insert user key");

        // Perform test

        assert_eq!(
            user_keys::get_by_id(&pool, &id)
                .await
                .expect("failed to get user key by id"),
            UserKey {
                id,
                user_id,
                encrypted_key
            }
        )
    }
}
