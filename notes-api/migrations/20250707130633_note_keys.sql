CREATE TABLE note_keys (
    id UUID PRIMARY KEY NOT NULL,
    note_id UUID NOT NULL,
    user_id UUID NOT NULL,
    encrypted_key BLOB NOT NULL,
    nonce BLOB NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
)