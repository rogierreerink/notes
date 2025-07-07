use uuid::Uuid;

pub struct NoteKey {
    pub id: Uuid,
    pub note_id: Uuid,
    pub user_id: Uuid,
    pub encrypted_key: String,
}
