pub mod app;
pub mod config;
pub mod database;
pub mod model;

mod bus;
mod server;

use app::App;

// GraphQL schema
pub mod schema {
    pub mod mutation;
    pub mod query;
    pub mod subscription;
    pub mod todo_item;

    use crate::app::App;

    use implementation::Impl;

    // Type alias for the complete TODO GraphQL schema
    pub type AppSchema = async_graphql::Schema<
        query::Query<Impl<App>>,
        mutation::Mutation<Impl<App>>,
        subscription::Subscription,
    >;
}

///
/// Run the application as a server
///
pub async fn run(app: App, port: Option<u16>) {
    server::serve(app, port).await;
}
