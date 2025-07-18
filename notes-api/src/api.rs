use std::sync::Arc;

use axum::{
    Router,
    routing::{post, put},
};

use crate::state::AppState;

pub mod users;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new().merge(
        Router::new()
            .route("/users", post(users::create_user))
            .route(
                "/users/{user_id}/password",
                put(users::create_or_update_user_password),
            )
            // .layer(
            //     ServiceBuilder::new()
            //         .layer(SetResponseHeaderLayer::if_not_present(
            //             header::ACCESS_CONTROL_ALLOW_METHODS,
            //             HeaderValue::from_static("GET, POST, OPTIONS"),
            //         ))
            //         .layer(SetResponseHeaderLayer::if_not_present(
            //             header::ACCESS_CONTROL_ALLOW_HEADERS,
            //             HeaderValue::from_static("content-type"),
            //         )),
            // )
            .with_state(state.clone()),
    )
}
