use async_graphql::http::GraphiQLSource;
use async_graphql::*;
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;

// Dopey example schema
struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // An infinite stream of hangup signals.
    let mut sig = signal(SignalKind::hangup())?;

    // Signal handler thread - send on signal_rx if SIGHUP received
    let (tx, signal_rx) = oneshot::channel();
    tokio::spawn(async move {
        sig.recv().await;
        let _ = tx.send(());
    });

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    tokio::select! {
        // Run the GraphQL server
        _ = axum::serve(TcpListener::bind("127.0.0.1:8000").await?, app) => {}

        // Listen for the HUP signal
        _ = signal_rx => {}
    }

    Ok(())
}
