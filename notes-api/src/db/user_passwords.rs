use sqlx::{SqliteExecutor, prelude::FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug, PartialEq)]
pub struct UserPassword {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_key_id: Uuid,
    pub password_hash: String,
}

pub async fn create<'e, E>(executor: E, user_password: &UserPassword) -> anyhow::Result<()>
where
    E: SqliteExecutor<'e>,
{
    sqlx::query(
        r#"
        INSERT INTO user_passwords (id, user_id, user_key_id, password_hash)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(&user_password.id)
    .bind(&user_password.user_id)
    .bind(&user_password.user_key_id)
    .bind(&user_password.password_hash)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: &Uuid) -> anyhow::Result<UserPassword>
where
    E: SqliteExecutor<'e>,
{
    Ok(sqlx::query_as(
        r#"
        SELECT id, user_id, user_key_id, password_hash
        FROM user_passwords
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(executor)
    .await?)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::db::user_passwords::{self, UserPassword};

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

        sqlx::query(
            r#"
            INSERT INTO user_keys (id, user_id, encrypted_key, nonce)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(&user_key_id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .execute(&pool)
        .await
        .expect("failed to insert user key");

        // Perform test

        let user_password = UserPassword {
            id: Uuid::new_v4(),
            user_id,
            user_key_id,
            password_hash: "5678".to_string(),
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

        sqlx::query(
            r#"
            INSERT INTO user_keys (id, user_id, encrypted_key, nonce)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(&user_key_id)
        .bind(&user_id)
        .bind(&encrypted_key)
        .bind(&nonce)
        .execute(&pool)
        .await
        .expect("failed to insert user key");

        let id = Uuid::new_v4();
        let password_hash = "5678".to_string();

        sqlx::query(
            r#"
            INSERT INTO user_passwords (id, user_id, user_key_id, password_hash)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&user_key_id)
        .bind(&password_hash)
        .execute(&pool)
        .await
        .expect("failed to insert user password");

        // Perform test

        assert_eq!(
            user_passwords::get_by_id(&pool, &id)
                .await
                .expect("failed to get user password by id"),
            UserPassword {
                id,
                user_id,
                user_key_id,
                password_hash
            }
        )
    }
}
