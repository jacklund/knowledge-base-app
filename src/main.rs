use async_graphql::http::GraphiQLSource;
use async_graphql::*;
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
// use dirs::*;
use crate::db::open_db;
use crate::graphql::{Mutation, Query};
use signal_hook::{
    consts::{SIGHUP, SIGINT},
    iterator::Signals,
};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

mod db;
mod error;
mod graphql;
mod object;
mod object_type;
mod tag;

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Register to catch SIGINT and SIGHUP
    let mut signals = Signals::new(&[SIGINT, SIGHUP])?;

    // Signal handler thread - send on signal_rx if signal received
    let (tx, signal_rx) = oneshot::channel();
    tokio::spawn(async move {
        // Wait for the next signal
        signals.forever().next();

        // Tell the main thread
        let _ = tx.send(());
    });

    // Build the schema
    let schema = Schema::build(Query, Mutation, EmptySubscription).finish();

    // Set up the routes
    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    // Open connection to DB
    open_db().await?;

    println!("GraphiQL IDE: http://localhost:8080");

    // Select between handling the connections and the signals
    tokio::select! {
        // Run the GraphQL server
        _ = axum::serve(TcpListener::bind("127.0.0.1:8080").await?, app) => {
        }

        // Listen for signals
        _ = signal_rx => {
        }
    }

    Ok(())
}
