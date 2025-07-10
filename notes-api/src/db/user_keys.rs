use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug, PartialEq)]
pub struct UserKeyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub encrypted_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

pub async fn create<'e, E>(executor: E, user_key: &UserKeyRow) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO user_keys (id, user_id, encrypted_key, nonce, salt)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(&user_key.id)
    .bind(&user_key.user_id)
    .bind(&user_key.encrypted_key)
    .bind(&user_key.nonce)
    .bind(&user_key.salt)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> anyhow::Result<UserKeyRow>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, user_id, encrypted_key, nonce, salt
        FROM user_keys
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(executor)
    .await?)
}

pub async fn get_by_user_id<'e, E>(executor: E, user_id: &Uuid) -> anyhow::Result<Vec<UserKeyRow>>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, user_id, encrypted_key, nonce, salt
        FROM user_keys
        WHERE user_id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
    .await?)
}

#[cfg(test)]
mod tests {
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::db::user_keys::{self, UserKeyRow};

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

        let user_key = UserKeyRow {
            id: Uuid::new_v4(),
            user_id,
            encrypted_key: vec![1, 2, 3, 4],
            nonce: vec![5, 6, 7, 8],
            salt: vec![4, 3, 2, 1],
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
        let encrypted_key = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];
        let salt = vec![4, 3, 2, 1];

        sqlx::query(
            r#"
            INSERT INTO user_keys (id, user_id, encrypted_key, nonce, salt)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .bind(&salt)
        .execute(&pool)
        .await
        .expect("failed to insert user key");

        // Perform test

        assert_eq!(
            user_keys::get_by_id(&pool, &id)
                .await
                .expect("failed to get user key by id"),
            UserKeyRow {
                id,
                user_id,
                encrypted_key,
                nonce,
                salt,
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

        let id = Uuid::new_v4();
        let encrypted_key = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];
        let salt = vec![4, 3, 2, 1];

        sqlx::query(
            r#"
            INSERT INTO user_keys (id, user_id, encrypted_key, nonce, salt)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .bind(&salt)
        .execute(&pool)
        .await
        .expect("failed to insert user key");

        // Perform test

        assert_eq!(
            user_keys::get_by_user_id(&pool, &user_id)
                .await
                .expect("failed to get user key by user id"),
            vec![UserKeyRow {
                id,
                user_id,
                encrypted_key,
                nonce,
                salt
            }]
        )
    }
}
