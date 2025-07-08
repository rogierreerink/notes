CREATE TABLE notes (
    id UUID PRIMARY KEY NOT NULL,
    encrypted_markdown BLOB NOT NULL,
    nonce BLOB NOT NULL
)