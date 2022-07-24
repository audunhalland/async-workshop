use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::Schema;
use entrait::*;

use crate::app::App;
use crate::bus::EventBus;
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
pub async fn serve(app: App, port: Option<u16>) {
    let port = port.unwrap_or(0);
    let schema: Schema<Query<Impl<App>>, Mutation<Impl<App>>, Subscription> =
        Schema::build(Query::default(), Mutation::default(), Subscription)
            .data(EventBus::new())
            .data(Impl::new(app))
            .finish();

    async fn graphql_handler(
        schema: axum::Extension<AppSchema>,
        req: GraphQLRequest,
    ) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    let graphql_playground = move || async move {
        axum::response::Html(playground_source(
            GraphQLPlaygroundConfig::new("/graphql")
                .subscription_endpoint(&format!("ws://localhost:{}", port)),
        ))
    };

    let app = axum::Router::new()
        .route(
            "/graphql",
            axum::routing::get(graphql_playground).post(graphql_handler),
        )
        .layer(axum::extract::Extension(schema));

    axum::Server::bind(&([0, 0, 0, 0], port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
