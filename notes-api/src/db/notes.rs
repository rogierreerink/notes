use uuid::Uuid;

pub struct Note {
    pub id: Uuid,
    pub encrypted_markdown: String,
}
