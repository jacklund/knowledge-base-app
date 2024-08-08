use async_graphql::http::GraphiQLSource;
use async_graphql::*;
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use surrealdb::sql::Kind;
// use dirs::*;
use crate::db::open_db;
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

    #[derive(Clone, Copy, Eq, Enum, PartialEq)]
    enum DataType {
        Bool,
        Int,
        Float,
        String,
    }

    impl From<&Kind> for DataType {
        fn from(kind: &Kind) -> Self {
            match kind {
                Kind::Bool => DataType::Bool,
                Kind::Int => DataType::Int,
                Kind::Float => DataType::Float,
                Kind::String => DataType::String,
                _ => unreachable!(),
            }
        }
    }

    impl From<&DataType> for Kind {
        fn from(datatype: &DataType) -> Self {
            match datatype {
                DataType::Bool => Kind::Bool,
                DataType::Int => Kind::Int,
                DataType::Float => Kind::Float,
                DataType::String => Kind::String,
            }
        }
    }

    #[derive(InputObject, SimpleObject)]
    struct ObjectTypeField {
        name: String,
        datatype: DataType,
        id: bool,
    }

    #[derive(SimpleObject)]
    struct ObjectType {
        name: String,
        fields: Vec<ObjectTypeField>,
    }

    impl From<&object_type::ObjectType> for ObjectType {
        fn from(o: &object_type::ObjectType) -> Self {
            Self {
                name: o.type_name.clone(),
                fields: o
                    .attributes
                    .iter()
                    .map(|(k, a)| ObjectTypeField {
                        name: k.clone(),
                        datatype: a.datatype().into(),
                        id: a.is_id_part(),
                    })
                    .collect(),
            }
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn get_object_types(&self) -> Result<Vec<ObjectType>> {
            Ok(object_type::ObjectType::get_all()
                .await?
                .iter()
                .map(|ot| ot.into())
                .collect())
        }
    }

    struct Mutation;

    #[Object]
    impl Mutation {
        async fn add_object_type<'a>(
            &self,
            ctx: &Context<'a>,
            name: String,
            fields: Vec<ObjectTypeField>,
        ) -> Result<ObjectType> {
            let mut object_type = object_type::ObjectType::new(&name);
            for field in fields {
                object_type
                    .add_attribute(
                        &field.name(ctx).await?,
                        field.datatype(ctx).await?.into(),
                        *field.id(ctx).await?,
                    )
                    .await?;
            }
            object_type.insert().await.unwrap();
            Ok((&object_type).into())
        }
    }

    // Build the schema
    let schema = Schema::build(Query, Mutation, EmptySubscription).finish();

    // Set up the routes
    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

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
