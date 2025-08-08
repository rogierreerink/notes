use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
    typed_header::TypedHeaderRejectionReason,
};
use josekit::jwk::Jwk;
use sqlx::SqlitePool;

use crate::{
    services,
    state::AppState,
    tokens::{self, TokenDecryptionError},
};

pub struct Auth(pub tokens::UserClaims);

impl<S> FromRequestParts<S> for Auth
where
    AuthState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract auth state
        let auth_state = AuthState::from_ref(state);

        // Extract authorization bearer
        let token = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|e| match e.reason() {
                TypedHeaderRejectionReason::Missing => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        // Get user claims from token cookie
        let user_claims =
            tokens::decrypt(token.token().as_bytes(), &auth_state.jwk).map_err(|e| match e {
                TokenDecryptionError::InvalidKey => StatusCode::UNAUTHORIZED,
                TokenDecryptionError::InvalidClaim(_) => StatusCode::BAD_REQUEST,
                TokenDecryptionError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        // Validate user session
        if !services::user_sessions::get(&auth_state.db, user_claims.session_id())
            .await
            .is_ok_and(|user_session| user_session.is_valid())
        {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(Auth(user_claims))
    }
}

pub struct AuthState {
    db: SqlitePool,
    jwk: Jwk,
}

impl FromRef<Arc<AppState>> for AuthState {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            db: input.db.clone(),
            jwk: input.jwk.clone(),
        }
    }
}
