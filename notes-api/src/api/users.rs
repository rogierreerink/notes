use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{extractors::auth::Auth, services, state::AppState, tokens};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    user: UserResponse,
    session: UserSessionResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: Uuid,
    username: String,
}

#[derive(Serialize)]
pub struct UserSessionResponse {
    token: String,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create user
    let user = services::users::User::new(payload.username);
    services::users::store(&mut *tx, &user).await.map_err(|e| {
        println!("failed to store user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create user session
    let user_session = services::user_sessions::UserSession::new(Duration::days(31));
    services::user_sessions::store(&mut *tx, &user_session, user.id())
        .await
        .map_err(|e| {
            println!("failed to store user session: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Commit database transaction
    tx.commit().await.map_err(|e| {
        println!("failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create temporary user key
    let user_key = services::user_keys::UserKey::new();

    // Wrap user session id, user id and user key in a JWT/JWE
    let user_claims = tokens::UserClaims::new(*user_session.id(), *user.id(), *user_key.key());
    let jwt = tokens::encrypt(&user_claims, &state.jwk).map_err(|e| {
        println!("failed to encrypt user claims: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateUserResponse {
            user: UserResponse {
                id: *user.id(),
                username: user.username().to_string(),
            },
            session: UserSessionResponse { token: jwt },
        }),
    ))
}

#[derive(Deserialize)]
pub struct CreateOrUpdateUserPasswordRequest {
    password: String,
}

pub async fn create_or_update_user_password(
    State(state): State<Arc<AppState>>,
    Auth(user_claims): Auth,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<CreateOrUpdateUserPasswordRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Authorize user
    if &user_id != user_claims.user_id() {
        println!("access denied");
        return Err(StatusCode::FORBIDDEN);
    }

    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Encrypt and store user key using user password
    let user_key = user_claims.user_key().into();
    services::user_keys::store_using_password(&mut *tx, &user_id, &user_key, &payload.password)
        .await
        .map_err(|e| {
            println!("failed to store user key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Store the user password
    let user_password =
        services::user_passwords::UserPassword::new(user_key.id(), &payload.password)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    services::user_passwords::store(&mut *tx, &user_password, &user_id)
        .await
        .map_err(|e| {
            println!("failed to store user password: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Commit database transaction
    tx.commit().await.map_err(|e| {
        println!("failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::CREATED)
}

pub async fn delete_user_session(
    State(state): State<Arc<AppState>>,
    Auth(user_claims): Auth,
    Path((user_id, session_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    // Authorize user
    if &user_id != user_claims.user_id() {
        println!("access denied");
        return Err(StatusCode::FORBIDDEN);
    }

    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Delete the user session
    services::user_sessions::delete(&mut *tx, &session_id)
        .await
        .map_err(|e| match e {
            services::Error::NotFound => {
                println!("access denied");
                StatusCode::FORBIDDEN
            }
            _ => {
                println!("failed to delete user session: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Commit database transaction
    tx.commit().await.map_err(|e| {
        println!("failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::OK)
}
