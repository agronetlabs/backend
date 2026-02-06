use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub type DbPool = Pool<Postgres>;

pub async fn connect() -> Result<DbPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var must be set");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
}
