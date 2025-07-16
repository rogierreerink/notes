use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::{TypedHeader, headers::Cookie, typed_header::TypedHeaderRejectionReason};

use crate::{
    state::AppState,
    tokens::{self, TokenDecryptionError},
};

pub struct UserClaims(pub tokens::UserClaims);

impl FromRequestParts<Arc<AppState>> for UserClaims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Extract cookies for HTTP headers
        let cookies = TypedHeader::<Cookie>::from_request_parts(parts, state)
            .await
            .map_err(|e| match e.reason() {
                TypedHeaderRejectionReason::Missing => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        // Extract token cookie from cookies
        let token_cookie = cookies
            .get("token")
            .ok_or_else(|| StatusCode::BAD_REQUEST)?;

        // Get user claims from token cookie
        let user_claims =
            tokens::decrypt(token_cookie.as_bytes(), &state.jwk).map_err(|e| match e {
                TokenDecryptionError::InvalidKey => StatusCode::UNAUTHORIZED,
                TokenDecryptionError::InvalidClaim(_) => StatusCode::BAD_REQUEST,
                TokenDecryptionError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(UserClaims(user_claims))
    }
}
