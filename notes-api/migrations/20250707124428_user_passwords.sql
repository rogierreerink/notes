CREATE TABLE user_passwords (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL,
    user_key_id UUID NOT NULL,
    password_hash CHAR(512) NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (user_key_id) REFERENCES user_keys (id) ON DELETE CASCADE
)