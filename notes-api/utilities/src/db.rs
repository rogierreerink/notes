use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn init_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect("sqlite::memory:")
        .await
        .expect("failed to create database");

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("failed to migrate database");

    pool
}
