CREATE TABLE user_sessions (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL,
    expiration_time TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
)