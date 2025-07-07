CREATE TABLE user_keys (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL,
    encrypted_key CHAR(512) NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
)