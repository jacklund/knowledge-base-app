use crate::object_type::ObjectType;
use std::sync::LazyLock;
use surrealdb::{
    engine::local::{Db, Mem},
    method::{Create, Delete, Select, Update},
    opt::IntoResource,
    Connection, Error, Result, Surreal,
};
use tokio::sync::Mutex;

// #[cfg(feature = "server")]
pub static DB_CONNECTION: LazyLock<Mutex<DbConnection>> =
    LazyLock::new(|| Mutex::new(DbConnection::new()));

const OBJECT_TYPE_TABLE: &str = "_object_type";

struct DbConnection {
    db: Surreal<Db>,
    connected: bool,
}

impl DbConnection {
    fn new() -> Self {
        Self {
            db: Surreal::init(),
            connected: false,
        }
    }

    async fn db(&mut self) -> Result<&mut Surreal<Db>> {
        if !self.connected {
            self.db.connect::<Mem>(()).await?;
            self.db.use_ns("test").use_db("test").await?;
            self.connected = true;
        }

        Ok(&mut self.db)
    }

    async fn select<R>(&mut self, resource: impl IntoResource<R>) -> Select<'_, Db, R> {
        self.db().await.unwrap().select(resource)
    }

    async fn create<R>(&mut self, resource: impl IntoResource<R>) -> Create<'_, Db, R> {
        self.db().await.unwrap().create(resource)
    }

    async fn update<R>(&mut self, resource: impl IntoResource<R>) -> Update<'_, Db, R> {
        self.db().await.unwrap().update(resource)
    }

    async fn delete<R>(&mut self, resource: impl IntoResource<R>) -> Delete<'_, Db, R> {
        self.db().await.unwrap().delete(resource)
    }
}

pub async fn get_object_types() -> Result<Vec<ObjectType>> {
    let mut db_connection = DB_CONNECTION.lock().await;
    db_connection.select(OBJECT_TYPE_TABLE).await.await
}

pub async fn new_object_type(object_type: &ObjectType) -> Result<Option<ObjectType>> {
    let mut db_connection = DB_CONNECTION.lock().await;
    db_connection
        .create((OBJECT_TYPE_TABLE, object_type.name()))
        .await
        .content(object_type)
        .await
}

pub async fn update_object_type(object_type: &ObjectType) -> Result<Option<ObjectType>> {
    let mut db_connection = DB_CONNECTION.lock().await;
    db_connection
        .update((OBJECT_TYPE_TABLE, object_type.name()))
        .await
        .content(object_type)
        .await
}

pub async fn delete_object_type(object_type: &ObjectType) -> Result<Option<ObjectType>> {
    let mut db_connection = DB_CONNECTION.lock().await;
    db_connection
        .delete((OBJECT_TYPE_TABLE, object_type.name()))
        .await
        .await
}
