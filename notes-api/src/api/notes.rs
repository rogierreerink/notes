use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{extractors::auth::Auth, services, state::AppState};

#[derive(Serialize, Deserialize)]
pub struct CreateOrUpdateNote {
    markdown: String,
}

pub async fn create_or_update_note(
    State(state): State<Arc<AppState>>,
    Auth(user_claims): Auth,
    Path(note_id): Path<Uuid>,
    Json(payload): Json<CreateOrUpdateNote>,
) -> Result<impl IntoResponse, StatusCode> {
    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update or create note
    let status = match services::notes::get_by_id(&mut *tx, &note_id).await {
        // Update existing note
        Ok(note) => {
            // Get and decrypt note key (decryption should not fail)
            let note_key = services::note_keys::get(&mut *tx, &note_id, user_claims.user_id())
                .await
                .map_err(|e| match e {
                    services::Error::NotFound => {
                        println!("access denied");
                        StatusCode::FORBIDDEN
                    }
                    _ => {
                        println!("failed to get note key: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                })?
                .decrypt(user_claims.user_key())
                .map_err(|e| {
                    println!("failed to decrypt note key: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            // Decrypt note (should not fail)
            let mut note = note.decrypt(note_key.key()).map_err(|e| {
                println!("failed to decrypt note: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Update note
            note.set_markdown(payload.markdown.clone());

            // Encrypt and store note
            services::notes::store(
                &mut *tx,
                note.encrypt(note_key.key()).map_err(|e| {
                    println!("failed to encrypt note: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
            )
            .await
            .map_err(|e| {
                println!("failed to store note: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            StatusCode::OK
        }

        // Create new note
        Err(services::Error::NotFound) => {
            let note = services::notes::DecryptedNote::new(note_id, payload.markdown.clone());
            let note_key = services::note_keys::DecryptedNoteKey::new();

            // Encrypt and store note
            services::notes::store(
                &mut *tx,
                note.encrypt(note_key.key()).map_err(|e| {
                    println!("failed to encrypt note: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
            )
            .await
            .map_err(|e| {
                println!("failed to store note: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Encrypt and store note key
            services::note_keys::store(
                &mut *tx,
                note_key.encrypt(user_claims.user_key()).map_err(|e| {
                    println!("failed to encrypt note key: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
                note.id(),
                user_claims.user_id(),
            )
            .await
            .map_err(|e| {
                println!("failed to store note key: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            StatusCode::CREATED
        }

        // Internal error
        Err(e) => {
            println!("failed to get note: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Commit database transaction
    tx.commit().await.map_err(|e| {
        println!("failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((status, Json(payload)))
}

#[derive(Serialize)]
pub struct GetNotesResponse {
    data: Vec<GetNotesResource>,
}

#[derive(Serialize)]
pub struct GetNotesResource {
    id: Uuid,
    title: Option<String>,
    // markdown: String,
}

pub async fn get_notes(
    State(state): State<Arc<AppState>>,
    Auth(user_claims): Auth,
) -> Result<impl IntoResponse, StatusCode> {
    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get note keys
    let mut notes = vec![];
    for link in services::note_keys::search(&mut *tx, user_claims.user_id())
        .await
        .map_err(|e| {
            println!("failed to search user keys: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    {
        // Get note
        let note = services::notes::get_by_id(&mut *tx, &link.note_id)
            .await
            .map_err(|e| {
                println!("failed to get note: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Decrypt note key (should not fail)
        let note_key = link.note_key.decrypt(user_claims.user_key()).map_err(|e| {
            println!("failed to decrypt note key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Decrypt note (should not fail)
        let note = note.decrypt(note_key.key()).map_err(|e| {
            println!("failed to decrypt note: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        notes.push(GetNotesResource {
            id: link.note_id,
            title: note.title().map(str::to_string),
            // markdown: note.markdown().to_string(),
        });
    }

    // Commit database transaction
    tx.commit().await.map_err(|e| {
        println!("failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(GetNotesResponse { data: notes }))
}

#[derive(Serialize)]
pub struct GetNoteResponse {
    id: Uuid,
    title: Option<String>,
    markdown: String,
}

pub async fn get_note(
    State(state): State<Arc<AppState>>,
    Auth(user_claims): Auth,
    Path(note_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get note
    let note = services::notes::get_by_id(&mut *tx, &note_id)
        .await
        .map_err(|e| match e {
            services::Error::NotFound => {
                println!("resource cannot be found");
                StatusCode::NOT_FOUND
            }
            _ => {
                println!("failed to get note: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Get note key
    let note_key = services::note_keys::get(&mut *tx, &note_id, user_claims.user_id())
        .await
        .map_err(|e| match e {
            services::Error::NotFound => {
                println!("access denied");
                StatusCode::FORBIDDEN
            }
            _ => {
                println!("failed to get note key: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Commit database transaction
    tx.commit().await.map_err(|e| {
        println!("failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Decrypt note key (should not fail)
    let note_key = note_key.decrypt(user_claims.user_key()).map_err(|e| {
        println!("failed to decrypt note key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Decrypt note (should not fail)
    let note = note.decrypt(note_key.key()).map_err(|e| {
        println!("failed to decrypt note: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(GetNoteResponse {
        id: note_id,
        title: note.title().map(str::to_string),
        markdown: note.markdown().to_string(),
    }))
}

pub async fn delete_note(
    State(state): State<Arc<AppState>>,
    Auth(user_claims): Auth,
    Path(note_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Start database transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get note
    let note = services::notes::get_by_id(&mut *tx, &note_id)
        .await
        .map_err(|e| match e {
            services::Error::NotFound => {
                println!("resource could not be found");
                StatusCode::NOT_FOUND
            }
            _ => {
                println!("failed to get note: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Get note key
    let note_key = services::note_keys::get(&mut *tx, &note_id, user_claims.user_id())
        .await
        .map_err(|e| match e {
            services::Error::NotFound => {
                println!("access denied");
                StatusCode::FORBIDDEN
            }
            _ => {
                println!("failed to get note key: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Decrypt note key (should not fail)
    let note_key = note_key.decrypt(user_claims.user_key()).map_err(|e| {
        println!("failed to decrypt note key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Decrypt note (should not fail)
    let note = note.decrypt(note_key.key()).map_err(|e| {
        println!("failed to decrypt note: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Delete note key
    services::note_keys::delete(&mut *tx, note.id(), user_claims.user_id())
        .await
        .map_err(|e| {
            println!("failed to delete note key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete note
    services::notes::delete(&mut *tx, note.id())
        .await
        .map_err(|e| {
            println!("failed to delete note: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Commit database transaction
    tx.commit().await.map_err(|e| {
        println!("failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::OK)
}
