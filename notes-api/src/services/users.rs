use sqlx::SqlitePool;

use crate::{db::users, services::User};

pub async fn register(db: &SqlitePool, user: &User) -> anyhow::Result<()> {
    let mut conn = db.acquire().await?;

    users::create(
        &mut *conn,
        &users::User {
            id: user.id,
            username: user.username.clone(),
        },
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::{db, services};

    #[tokio::test]
    async fn register_user() {
        let pool = init_db().await;

        let user = services::User {
            id: Uuid::new_v4(),
            username: "test".to_string(),
        };

        services::users::register(&pool, &user)
            .await
            .expect("failed to register user");

        assert_eq!(
            db::users::get_by_id(&pool, &user.id)
                .await
                .expect("failed to get user by id"),
            db::users::User {
                id: user.id,
                username: user.username
            }
        )
    }
}
