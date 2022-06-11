use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::Schema;

use crate::bus::EventBus;
use crate::repository::Repository;
use crate::schema::mutation::Mutation;
use crate::schema::query::Query;
use crate::schema::subscription::Subscription;
use crate::schema::AppSchema;

///
/// Start a web server providing the /graphql endpoint plus a playground.
///
/// The server runs as long as its future is polled by the executor.
/// The server is a future that never completes.
///
pub async fn serve(port: Option<u16>, pg_pool: sqlx::PgPool) {
    let port = port.unwrap_or(0);
    let schema = Schema::build(Query, Mutation, Subscription)
        .data(Repository::new(pg_pool))
        .data(EventBus::new())
        .finish();

    async fn graphql_handler(
        schema: axum::Extension<AppSchema>,
        req: GraphQLRequest,
    ) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    let graphql_playground2 = move || async move {
        axum::response::Html(playground_source(
            GraphQLPlaygroundConfig::new("/graphql")
                .subscription_endpoint(&format!("ws://localhost:{}", port)),
        ))
    };

    let app = axum::Router::new()
        .route(
            "/graphql",
            axum::routing::get(graphql_playground2).post(graphql_handler),
        )
        .layer(axum::extract::Extension(schema));

    axum::Server::bind(&([0, 0, 0, 0], port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
