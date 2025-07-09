use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db;

pub async fn create_user(db: &SqlitePool, user_id: &Uuid, username: &String) -> anyhow::Result<()> {
    let mut conn = db.acquire().await?;

    db::users::create(
        &mut *conn,
        &db::users::User {
            id: *user_id,
            username: username.clone(),
        },
    )
    .await?;

    Ok(())
}

// pub async fn create_password(
//     db: &SqlitePool,
//     user_id: &Uuid,
//     password: &String,
// ) -> anyhow::Result<()> {
//     let mut conn = db.acquire().await?;

//     Ok(())
// }

#[cfg(test)]
mod tests {
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::{db, services};

    #[tokio::test]
    async fn create_user() {
        let pool = init_db().await;

        let id = Uuid::new_v4();
        let username = "test".to_string();

        services::users::create_user(&pool, &id, &username)
            .await
            .expect("failed to create user");

        assert_eq!(
            db::users::get_by_id(&pool, &id)
                .await
                .expect("failed to get user by id"),
            db::users::User { id, username }
        )
    }
}
