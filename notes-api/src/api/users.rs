use chrono::Duration;
use uuid::Uuid;

use crate::services;

async fn create_user() {
    // Create a new user
    // let user = services::users::User::new("test");
    // services::users::store(&mut *tx, &user).await;

    // Create a temporary user key
    // let user_key = services::user_keys::UserKey::new();

    // Create a new user session
    // let user_session =
    //     services::user_sessions::UserSession::new(Some(Duration::days(31)));
    // services::user_sessions::store(&mut *tx, &user_session).await;

    // Wrap the user id, user key and user session id in a JWT/JWE

    // Store the user session JWE in set-cookie response header
}

async fn create_user_password() {
    // Validate the user session
    // Store the user password
}

async fn create_user_session() {
    // Authenticate the user
    // Create a new user session
    // Wrap the user id, user key and user session id in a JWT/JWE
    // Store the user session JWE in set-cookie response header
}
