use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
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
    id: Uuid,
    username: String,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create user
    let user = services::users::User::new(payload.username);
    services::users::store(&mut *tx, &user).await.map_err(|e| {
        println!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create user session
    let user_session = services::user_sessions::UserSession::new(Duration::days(31));
    services::user_sessions::store(&mut *tx, &user_session, user.id())
        .await
        .map_err(|e| {
            println!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Commit database transaction
    tx.commit().await.map_err(|e| {
        println!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create temporary user key
    let user_key = services::user_keys::UserKey::new();

    // Wrap user session id, user id and user key in a JWT/JWE
    let user_claims = tokens::UserClaims::new(*user_session.id(), *user.id(), *user_key.key());
    let jwt = tokens::encrypt(&user_claims, &state.jwk).map_err(|e| {
        println!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create JWT cookie
    let mut jwt_cookie = cookie::Cookie::new("token", &jwt);
    jwt_cookie.set_http_only(true);
    jwt_cookie.set_secure(true);
    jwt_cookie.set_same_site(cookie::SameSite::Strict);

    // Create response headers
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        jwt_cookie.to_string().parse().map_err(|e| {
            println!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    );

    Ok((
        StatusCode::CREATED,
        headers,
        Json(CreateUserResponse {
            id: *user.id(),
            username: user.username().to_string(),
        }),
    ))
}

#[derive(Deserialize)]
pub struct CreateUserPasswordRequest {
    password: String,
}

#[derive(Serialize)]
pub struct CreateUserPasswordResponse {
    id: Uuid,
    password: String,
}

pub async fn create_user_password(
    State(state): State<Arc<AppState>>,
    Path((user_id, password_id)): Path<(Uuid, Uuid)>,
    Auth(user_claims): Auth,
    Json(payload): Json<CreateUserPasswordRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate user access
    if &user_id != user_claims.user_id() {
        println!("resource forbidden");
        return Err(StatusCode::FORBIDDEN);
    }

    // Store the user password

    Ok((StatusCode::CREATED,))
}

pub async fn create_user_session() {
    // Authenticate the user
    // Create a new user session
    // Wrap the user id, user key and user session id in a JWT/JWE
    // Store the user session JWE in set-cookie response header
}
