use josekit::jwk::Jwk;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub struct AppState {
    pub db: SqlitePool,
    pub jwk: Jwk,
}

impl AppState {
    pub async fn init() -> anyhow::Result<Self> {
        // Setup database
        let db = Self::init_db().await?;

        // Setup JWT key
        let jwk = Self::init_jwk()?;

        Ok(Self { db, jwk })
    }

    async fn init_db() -> anyhow::Result<SqlitePool> {
        // Create database connection pool
        let db = SqlitePoolOptions::new()
            .max_connections(4)
            .connect("sqlite::memory:")
            .await?;

        // Run database migrations
        sqlx::migrate!("./migrations").run(&db).await?;

        Ok(db)
    }

    fn init_jwk() -> anyhow::Result<Jwk> {
        Ok(Jwk::generate_oct_key(32)?)
    }
}
