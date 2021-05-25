pub mod config;
pub mod model;
pub mod repository;

mod bus;
mod server;

// GraphQL schema
pub mod schema {
    pub mod mutation;
    pub mod query;
    pub mod subscription;
    pub mod todo_item;

    // Type alias for the complete TODO GraphQL schema
    pub type AppSchema =
        async_graphql::Schema<query::Query, mutation::Mutation, subscription::Subscription>;
}

///
/// Run the application as a server
///
pub async fn run(port: Option<u16>, pg_pool: sqlx::PgPool) {
    server::serve(port, pg_pool).await;
}
