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
use surrealdb::sql::{Array, Id, Object as DbObject, Value};
use surrealdb::Surreal;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::db::generic_object::{AttributeValue, GenericObject};
use crate::db::{open_db, parse_array, read_all, DB};

mod db;

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
    let mut signals = Signals::new(&[SIGINT, SIGHUP])?;

    // Signal handler thread - send on signal_rx if SIGHUP received
    let (tx, signal_rx) = oneshot::channel();
    tokio::spawn(async move {
        signals.forever();
        let _ = tx.send(());
    });

    // Open the DB
    open_db().await?;
    let mut jack = GenericObject::new("person").add_string_attribute("name", "Jack");
    jack.insert().await?;
    let mut fred = GenericObject::new("person")
        .add_string_attribute("name", "Fred")
        .add_int_attribute("age", 62);
    fred.insert().await?;
    let objects = read_all("person").await?;
    for object in objects {
        println!("{:?}", object);
    }
    let mut book = GenericObject::new("book")
        .add_string_attribute("name", "Fred's book")
        .add_string_attribute("author", "Fred");
    println!("book before = {:?}", book);
    book.insert().await?;
    println!("book after = {:?}", book);
    DB.query("define field in on table wrote type record<person>")
        .query("define field out on table wrote type record<book>")
        .await?;
    let mut response = DB
        .query("select id from person where name = \"Fred\"")
        .await?;
    match response.take::<Value>(0)?.first() {
        Value::Object(object) => {
            println!("id = {:?}", object.get("id").unwrap().clone().as_string())
        }
        _ => unreachable!(),
    }

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
