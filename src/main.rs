use async_graphql::http::GraphiQLSource;
use async_graphql::{dynamic::*, Value};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
// use dirs::*;
use crate::db::open_db;
use crate::object_type::ObjectType;
use signal_hook::{
    consts::{SIGHUP, SIGINT},
    iterator::Signals,
};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

mod db;
mod error;
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

    // Open connection to DB
    open_db().await?;

    // Define ObjectType type
    let object_type = Object::new("ObjectType")
        .description("An object type")
        .field(Field::new("name", TypeRef::named_nn(TypeRef::ID), |ctx| {
            FieldFuture::new(async move {
                let object_type = ctx.parent_value.try_downcast_ref::<ObjectType>()?;
                Ok(Some(Value::from(object_type.type_name().to_owned())))
            })
        }));

    // Define Query Root
    let query_root = Object::new("Query").field(Field::new(
        "getObjectTypes",
        TypeRef::named_list(object_type.type_name()),
        |_ctx| {
            FieldFuture::new(async move {
                let object_types = ObjectType::get_all().await?;
                Ok(Some(FieldValue::list(
                    object_types.into_iter().map(FieldValue::owned_any),
                )))
            })
        },
    ));

    // Build the schema
    let schema = Schema::build(query_root.type_name(), None, None)
        .register(object_type)
        .register(query_root)
        .finish()?;

    // Set up the routes
    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    // Select between handling the connections and the signals
    tokio::select! {
        // Run the GraphQL server
        _ = axum::serve(TcpListener::bind("127.0.0.1:8000").await?, app) => {
        }

        // Listen for signals
        _ = signal_rx => {
        }
    }

    Ok(())
}
