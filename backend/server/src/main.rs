use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
    routing::get,
    Router,
    Server,
};
use db::DB;
use schema::{MonopolySchema, MutationRoot, QueryRoot};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

mod db;
mod model;
mod schema;

async fn graphql_handler(
    schema: Extension<MonopolySchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() {
    println!("Starting server (Axum 0.6)...");

    let db = DB::init().await;
    
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db)
        .finish();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running on http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
