use async_graphql::http::GraphiQLSource;
use async_graphql::*;
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
// use dirs::*;
use signal_hook::{
    consts::{SIGHUP, SIGINT},
    iterator::Signals,
};
use surrealdb::engine::local::{Db, Mem, RocksDb};
use surrealdb::sql::{Array, Object as DbObject, Value};
use surrealdb::Surreal;
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

// Open the DB
async fn open_db() -> surrealdb::Result<Surreal<Db>> {
    // let mut db_dir = data_local_dir().unwrap();
    // db_dir.push("knowledge_repo");
    // Surreal::new::<RocksDb>(db_dir).await
    Surreal::new::<Mem>(()).await
}

fn parse_value(value: Value) -> DbObject {
    match value {
        Value::Object(object) => object,
        _ => {
            println!("Whoops: value = {:?}", value);
            std::process::exit(1);
        }
    }
}

fn parse_array(array: Array) -> Vec<DbObject> {
    if array.len() == 1 {
        parse_array(array)
    } else {
        array.iter().map(|v| parse_value(v.clone())).collect()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // An infinite stream of hangup signals.
    let mut signals = Signals::new(&[SIGINT, SIGHUP])?;

    // Signal handler thread - send on signal_rx if SIGHUP received
    let (tx, signal_rx) = oneshot::channel();
    tokio::spawn(async move {
        signals.forever();
        let _ = tx.send(());
    });

    // Open the DB
    let db = open_db().await?;
    db.use_ns("test").use_db("test").await?;
    db.query("insert into person [{name: \"Jack\"}, {name: \"Fred\", age: 62}]")
        .await?;
    let mut response = db.query("select * from person").await?;
    let result: Value = response.take(0)?;
    match result {
        Value::Array(array) => {
            for obj in parse_array(array).iter() {
                for (key, value) in obj.iter() {
                    println!("{}: {}", key, value);
                }
            }
        }
        _ => println!("Whoops!, result = {:?}", result),
    };

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
