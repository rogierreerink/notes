use chrono::{DateTime, Duration, Utc};
use sqlx::SqliteExecutor;
use uuid::Uuid;

use crate::{db, services};

#[derive(Debug, PartialEq)]
pub struct UserSession {
    id: Uuid,
    expiration_time: Option<DateTime<Utc>>,
}

impl UserSession {
    pub fn new(validity: Duration) -> Self {
        Self {
            id: Uuid::new_v4(),
            expiration_time: Some(Utc::now() + validity),
        }
    }

    pub fn new_persistent() -> Self {
        Self {
            id: Uuid::new_v4(),
            expiration_time: None,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn is_valid(&self) -> bool {
        self.expiration_time
            .is_none_or(|expiration_time| expiration_time > Utc::now())
    }
}

pub async fn store<'e, E>(
    executor: E,
    user_session: &UserSession,
    user_id: &Uuid,
) -> services::Result<()>
where
    E: SqliteExecutor<'e>,
{
    // Store the user session
    db::user_sessions::create(
        executor,
        &db::user_sessions::UserSessionRow {
            id: user_session.id,
            user_id: *user_id,
            expiration_time: user_session.expiration_time,
        },
    )
    .await?;

    Ok(())
}

pub async fn get<'e, E>(executor: E, id: &Uuid) -> services::Result<UserSession>
where
    E: SqliteExecutor<'e>,
{
    // Get the user session
    Ok(db::user_sessions::get_by_id(executor, id)
        .await
        .map(|row| UserSession {
            id: row.id,
            expiration_time: row.expiration_time,
        })?)
}

pub async fn delete<'e, E>(executor: E, session_id: &Uuid) -> services::Result<()>
where
    E: SqliteExecutor<'e>,
{
    // Delete user session from database
    db::user_sessions::delete_by_id(executor, session_id).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use utilities::db::init_db;
    use uuid::Uuid;

    use crate::{db, services};

    #[tokio::test]
    async fn store() {
        let pool = init_db().await;

        // Populate database

        let user_id = Uuid::new_v4();

        db::users::create(
            &pool,
            &db::users::UserRow {
                id: user_id,
                username: "test".to_string(),
            },
        )
        .await
        .expect("failed to create user");

        // Perform test

        let user_session = services::user_sessions::UserSession::new(Duration::days(31));

        services::user_sessions::store(&pool, &user_session, &user_id)
            .await
            .expect("failed to store user session");

        assert_eq!(
            db::user_sessions::get_by_id(&pool, &user_session.id)
                .await
                .expect("failed to get user session by id"),
            db::user_sessions::UserSessionRow {
                id: user_session.id,
                user_id,
                expiration_time: user_session.expiration_time
            }
        )
    }

    #[tokio::test]
    async fn get() {
        let pool = init_db().await;

        // Populate database

        let user_id = Uuid::new_v4();

        db::users::create(
            &pool,
            &db::users::UserRow {
                id: user_id,
                username: "test".to_string(),
            },
        )
        .await
        .expect("failed to create user");

        // Perform test

        let user_session = services::user_sessions::UserSession::new(Duration::days(31));

        services::user_sessions::store(&pool, &user_session, &user_id)
            .await
            .expect("failed to store user session");

        assert_eq!(
            services::user_sessions::get(&pool, &user_session.id)
                .await
                .expect("failed to get user session"),
            user_session
        );
    }

    #[tokio::test]
    async fn is_valid_non_persistent() {
        assert!(services::user_sessions::UserSession::new(Duration::days(31)).is_valid());
    }

    #[tokio::test]
    async fn is_valid_persistent() {
        assert!(services::user_sessions::UserSession::new_persistent().is_valid());
    }

    #[tokio::test]
    async fn is_invalid() {
        assert!(!services::user_sessions::UserSession::new(Duration::days(-1)).is_valid());
    }
}
