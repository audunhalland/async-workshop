//!
//! Code for managing test databases to be used in integration tests.
//!

use async_workshop::app::GetPgPoolMock;
use dotenv::dotenv;
use sqlx::Connection;

use unimock::MockFn;

///
/// Create a "fresh" Postgres database for running tests with.
///
/// In Rust, test run in parallel. To keep them isolated, they should use different
/// databases. A solution to this is to create one database per test thread,
/// database name derived from the thread name.
///
/// Returns an unimock spy on the application, with `GetPgPool` implemented
/// as getting a connection to the specific test database.
///
pub async fn create_test_db() -> unimock::Unimock {
    let db_name = format!("test_db_{}", std::thread::current().name().unwrap());
    let mut url = database_server_url();
    let mut connection = sqlx::PgConnection::connect(url.as_str()).await.unwrap();

    sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{}""#, db_name))
        .execute(&mut connection)
        .await
        .expect("failed to drop");

    sqlx::query(&format!(r#"CREATE DATABASE "{}""#, db_name))
        .execute(&mut connection)
        .await
        .expect("failed creating test database");

    url.set_path(&db_name);

    let pg_pool = sqlx::PgPool::connect(url.as_str())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to migrate");

    unimock::Unimock::new_partial(
        GetPgPoolMock
            .each_call(unimock::matching!())
            .returns(pg_pool)
    )
}

// Load DATABASE_URL, but strip away its path, i.e. /database_name
fn database_server_url() -> url::Url {
    // (re)load the .env file
    dotenv().ok();

    let mut url: url::Url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .parse()
        .expect("malformed DATABASE_URL");

    if let Ok(mut path) = url.path_segments_mut() {
        path.clear();
    }

    url
}
