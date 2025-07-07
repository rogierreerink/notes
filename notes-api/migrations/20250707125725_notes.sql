CREATE TABLE notes (
    id UUID PRIMARY KEY NOT NULL,
    encrypted_markdown TEXT NOT NULL UNIQUE
)