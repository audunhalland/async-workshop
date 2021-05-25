use async_workshop::config::Config;

#[tokio::main]
async fn main() {
    // Initialize the tracing logger
    tracing_subscriber::fmt::init();

    let config = Config {
        db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    };
    let pg_pool = sqlx::PgPool::connect(&config.db_url)
        .await
        .expect("Failed to connect to database");

    // The `migrate!` macro _embeds_ the migration files into the resulting binary,
    // so there is no need to worry about the filesystem during runtime
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to migrate");

    async_workshop::run(Some(8000), pg_pool).await;
}
