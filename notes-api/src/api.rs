use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::state::AppState;

pub mod auth;
pub mod notes;
pub mod users;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new().merge(
        Router::new()
            .route("/auth", post(auth::create_user_session_token))
            .route("/users", post(users::create_user))
            .route(
                "/users/{user_id}/password",
                put(users::create_or_update_user_password),
            )
            .route(
                "/users/{user_id}/sessions/{session_id}",
                delete(users::delete_user_session),
            )
            .route("/notes", get(notes::get_notes))
            .route("/notes/{note_id}", get(notes::get_note))
            .route("/notes/{note_id}", put(notes::create_or_update_note))
            .route("/notes/{note_id}", delete(notes::delete_note))
            .with_state(state.clone()),
    )
}
