use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug, PartialEq)]
pub struct UserPasswordRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_key_id: Uuid,
    pub hash: Vec<u8>,
    pub salt: Vec<u8>,
}

pub async fn create<'e, E>(executor: E, user_password: &UserPasswordRow) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO user_passwords (id, user_id, user_key_id, hash, salt)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(&user_password.id)
    .bind(&user_password.user_id)
    .bind(&user_password.user_key_id)
    .bind(&user_password.hash)
    .bind(&user_password.salt)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> anyhow::Result<UserPasswordRow>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, user_id, user_key_id, hash, salt
        FROM user_passwords
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(executor)
    .await?)
}

pub async fn get_by_user_id<'e, E>(executor: E, user_id: &Uuid) -> anyhow::Result<UserPasswordRow>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, user_id, user_key_id, hash, salt
        FROM user_passwords
        WHERE user_id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_one(executor)
    .await?)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::db::user_passwords::{self, UserPasswordRow};

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

        let user_key_id = Uuid::new_v4();
        let encrypted_key = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];
        let salt = vec![4, 3, 2, 1];

        sqlx::query(
            r#"
            INSERT INTO user_keys (id, user_id, encrypted_key, nonce, salt)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&user_key_id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .bind(&salt)
        .execute(&pool)
        .await
        .expect("failed to insert user key");

        // Perform test

        let user_password = UserPasswordRow {
            id: Uuid::new_v4(),
            user_id,
            user_key_id,
            hash: vec![1, 2, 3, 4],
            salt: vec![4, 3, 2, 1],
        };

        user_passwords::create(&pool, &user_password)
            .await
            .expect("failed to create user password");

        assert_eq!(
            user_passwords::get_by_id(&pool, &user_password.id)
                .await
                .expect("failed to get user password by id"),
            user_password,
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

        let user_key_id = Uuid::new_v4();
        let encrypted_key = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];
        let salt = vec![4, 3, 2, 1];

        sqlx::query(
            r#"
            INSERT INTO user_keys (id, user_id, encrypted_key, nonce, salt)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&user_key_id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .bind(&salt)
        .execute(&pool)
        .await
        .expect("failed to insert user key");

        let id = Uuid::new_v4();
        let hash = vec![1, 2, 3, 4];
        let salt = vec![4, 3, 2, 1];

        sqlx::query(
            r#"
            INSERT INTO user_passwords (id, user_id, user_key_id, hash, salt)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&user_key_id)
        .bind(&hash)
        .bind(&salt)
        .execute(&pool)
        .await
        .expect("failed to insert user password");

        // Perform test

        assert_eq!(
            user_passwords::get_by_id(&pool, &id)
                .await
                .expect("failed to get user password by id"),
            UserPasswordRow {
                id,
                user_id,
                user_key_id,
                hash,
                salt
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

        let user_key_id = Uuid::new_v4();
        let encrypted_key = vec![1, 2, 3, 4];
        let nonce = vec![5, 6, 7, 8];
        let salt = vec![4, 3, 2, 1];

        sqlx::query(
            r#"
            INSERT INTO user_keys (id, user_id, encrypted_key, nonce, salt)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&user_key_id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .bind(&salt)
        .execute(&pool)
        .await
        .expect("failed to insert user key");

        let id = Uuid::new_v4();
        let hash = vec![1, 2, 3, 4];
        let salt = vec![4, 3, 2, 1];

        sqlx::query(
            r#"
            INSERT INTO user_passwords (id, user_id, user_key_id, hash, salt)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&user_key_id)
        .bind(&hash)
        .bind(&salt)
        .execute(&pool)
        .await
        .expect("failed to insert user password");

        // Perform test

        assert_eq!(
            user_passwords::get_by_user_id(&pool, &user_id)
                .await
                .expect("failed to get user password by user id"),
            UserPasswordRow {
                id,
                user_id,
                user_key_id,
                hash,
                salt
            }
        )
    }
}
