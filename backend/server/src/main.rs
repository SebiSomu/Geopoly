use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
    routing::get,
    Router,
    Server,
};
use database::DB;
use schema::{GeopolySchema, MutationRoot, QueryRoot};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

mod database;
mod model;
mod schema;

async fn graphql_handler(
    schema: Extension<GeopolySchema>,
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
    
    let (message_sender, _message_receiver) = tokio::sync::broadcast::channel::<crate::model::ChatMessage>(100);
    
    let schema = Schema::build(QueryRoot, MutationRoot, schema::SubscriptionRoot)
        .data(db)
        .data(message_sender)
        .finish();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route_service("/graphql/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running on http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
