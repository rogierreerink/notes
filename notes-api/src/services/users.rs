use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db;

pub struct User {
    id: Uuid,
    username: String,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
        }
    }
}

pub async fn store(db: &SqlitePool, user: &User) -> anyhow::Result<()> {
    let mut conn = db.acquire().await?;

    // Store the user
    db::users::create(
        &mut *conn,
        &db::users::UserRow {
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

    use crate::{db, services};

    #[tokio::test]
    async fn create() {
        let pool = init_db().await;

        let user = services::users::User::new("test".to_string());

        services::users::store(&pool, &user)
            .await
            .expect("failed to store user");

        assert_eq!(
            db::users::get_by_id(&pool, &user.id)
                .await
                .expect("failed to get user by id"),
            db::users::UserRow {
                id: user.id,
                username: user.username
            }
        )
    }
}
