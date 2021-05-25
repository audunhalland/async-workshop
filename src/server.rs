use std::convert::Infallible;

use async_graphql_warp::graphql_subscription;
use warp::http::StatusCode;
use warp::{http::Response as HttpResponse, Filter, Rejection};

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::Schema;

use crate::bus::EventBus;
use crate::repository::Repository;
use crate::schema::mutation::Mutation;
use crate::schema::query::Query;
use crate::schema::subscription::Subscription;
use crate::schema::AppSchema;

///
/// Execute a GraphQL request on our AppSchema and convert the response
/// to a warp-compatible data type.
///
/// The argument to this function is a tuple because of how the `async_graphql_warp` filter works.
/// It's possible to refactor this to take two separate arguments instead.
///
async fn execute_graphql(
    (schema, request): (AppSchema, async_graphql::Request),
) -> Result<async_graphql_warp::Response, Infallible> {
    let response = schema.execute(request).await;
    Ok(async_graphql_warp::Response::from(response))
}

///
/// Endpoints in warp implement the warp::Filter trait.
/// Filters get composed of smaller building blocks using various combinators.
///
/// A "top level" HTTP endpoint normally implements `Filter<Extract = impl warp::Reply, Error = warp::Rejection>`.
///
mod filters {
    use super::*;

    pub fn post_graphql(
        schema: AppSchema,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path("graphql"))
            .and(warp::path::end())
            // GraphQL request parser filter:
            // `and` filters statically builds up `Extract` argument lists.
            // The `Extract` of the following filter is a tuple of `(AppSchema, Request)`.
            // The `Extract` of the preceding filters are empty tuples...
            .and(async_graphql_warp::graphql(schema))
            // ...so the `execute_graphql` function accepts this _one tuple_ as its argument.
            // The `.and_then` combinator is an asynchronous mapper, in this
            // case mapping the Extract type to the filter's output trait, `warp::Reply`.
            .and_then(execute_graphql)
            .with(warp::trace::named("graphql"))
    }

    pub fn get_graphql_playground(
        websocket_port: u16,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get().and(warp::path::end()).map(move || {
            HttpResponse::builder()
                .header("content-type", "text/html")
                .body(playground_source(
                    GraphQLPlaygroundConfig::new("/graphql")
                        .subscription_endpoint(&format!("ws://localhost:{}", websocket_port)),
                ))
        })
    }
}

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

    let routes = filters::post_graphql(schema.clone())
        .or(graphql_subscription(schema))
        .or(filters::get_graphql_playground(port))
        .recover(|err: Rejection| async move {
            if let Some(async_graphql_warp::BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        });

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
