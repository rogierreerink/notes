use sqlx::SqliteExecutor;
use uuid::Uuid;

use crate::{db, services};

#[derive(Debug, PartialEq)]
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

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

pub async fn store<'e, E>(executor: E, user: &User) -> services::Result<()>
where
    E: SqliteExecutor<'e>,
{
    // Store the user
    db::users::create(
        executor,
        &db::users::UserRow {
            id: user.id,
            username: user.username.clone(),
        },
    )
    .await?;

    Ok(())
}

pub async fn get_by_username<'e, E>(executor: E, username: &str) -> services::Result<User>
where
    E: SqliteExecutor<'e>,
{
    // Get the user
    let user_row = db::users::get_by_username(executor, username).await?;

    Ok(User {
        id: user_row.id,
        username: user_row.username,
    })
}

#[cfg(test)]
mod tests {
    use utilities::db::init_db;

    use crate::{db, services};

    #[tokio::test]
    async fn store() {
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

    #[tokio::test]
    async fn get_by_username() {
        let pool = init_db().await;

        let user = services::users::User::new("test".to_string());

        services::users::store(&pool, &user)
            .await
            .expect("failed to store user");

        assert_eq!(
            services::users::get_by_username(&pool, &user.username)
                .await
                .expect("failed to get user by username"),
            user
        )
    }
}
