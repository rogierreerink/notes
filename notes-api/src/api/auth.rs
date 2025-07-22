use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{services, state::AppState, tokens};

#[derive(Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum CreateUserSessionRequest {
    Password { username: String, password: String },
}

#[derive(Serialize)]
pub struct CreateUserSessionResponse {
    id: Uuid,
}

pub async fn create_user_session_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserSessionRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Authenticate user and get user key
    let (user, user_key) = match payload {
        CreateUserSessionRequest::Password { username, password } => {
            let user = services::users::get_by_username(&mut *tx, &username)
                .await
                .map_err(|e| {
                    println!("failed to get user: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            let user_password = services::user_passwords::get_by_user_id(&mut *tx, user.id())
                .await
                .map_err(|e| {
                    println!("failed to get user password: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            if !user_password.verify(&password).map_err(|e| {
                println!("failed to verify user password: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })? {
                println!("incorrect password");
                return Err(StatusCode::UNAUTHORIZED);
            }

            let user_key = services::user_keys::get_using_password(
                &mut *tx,
                user_password.user_key_id(),
                &password,
            )
            .await
            .map_err(|e| {
                println!("failed to get user key: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            (user, user_key)
        }
    };

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

    // Wrap user session id, user id and user key in a JWT/JWE
    let user_claims = tokens::UserClaims::new(*user_session.id(), *user.id(), *user_key.key());
    let jwt = tokens::encrypt(&user_claims, &state.jwk).map_err(|e| {
        println!("failed to encrypt user claims: {}", e);
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
            println!("failed to parse jwt into header value: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(CreateUserSessionResponse { id: *user.id() }),
    ))
}
