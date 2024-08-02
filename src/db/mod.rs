use crate::db::generic_object::{GenericObject, TableId};
use crate::schema::SCHEMAS;
use std::sync::LazyLock;
// use surrealdb::engine::any::Any;
use surrealdb::engine::local::{Db, Mem, RocksDb};
// use surrealdb::engine::remote::ws::Ws;
use surrealdb::sql::{Array, Object as DbObject, Value};
use surrealdb::{Result, Surreal};

pub mod generic_object;

pub static DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());
// pub static DB: LazyLock<Surreal<Any>> = LazyLock::new(|| Surreal::init());

// Open the DB
pub async fn open_db() -> Result<()> {
    // let mut db_dir = data_local_dir().unwrap();
    // db_dir.push("knowledge_repo");
    // DB.connect::<RocksDb>(db_dir).await

    // Use in-mem db for now
    open_test_db().await
}

// Open the DB
pub async fn open_test_db() -> Result<()> {
    DB.connect::<Mem>(()).await?;
    // DB.connect("http://localhost:8000").await?;
    DB.use_ns("test").use_db("test").await?;
    Ok(())
}

fn parse_value(value: Value) -> DbObject {
    match value {
        Value::Object(object) => object,
        _ => {
            unreachable!()
        }
    }
}

pub fn parse_array(array: Array) -> Vec<DbObject> {
    array.iter().map(|v| parse_value(v.clone())).collect()
}

/// Read all rows in a table
pub async fn read_all(table: &str) -> Result<Vec<GenericObject>> {
    // Get a copy of the schema for the object
    let schema = {
        let mut schemas = SCHEMAS.lock().unwrap();
        let lock = schemas.get(table).await?;
        let schema = lock.read().unwrap().clone();
        schema
    };

    // Since we're reading a generic object, we need to parse
    // it into the object ourselves
    let mut ret = Vec::new();
    let mut response = DB.query(format!("select * from {table}")).await?;

    // Read as array of Value
    let result: Value = response.take(0)?;
    match result {
        Value::Array(array) => {
            for obj in parse_array(array).iter_mut() {
                // Grab the table ID, and set it in the object
                let id: TableId = obj.remove("id").unwrap().into();
                let mut object = GenericObject::new(&id.table_name()).await?;
                object.set_table_id(id);

                // Load the attributes in the same order as in the schema
                for attr in schema.attributes() {
                    if let Some((key, value)) = obj.iter().find(|(k, _)| **k == attr.name()) {
                        object = object.add_kv_attribute(key, value.clone().into()).await?;
                    }
                }
                ret.push(object);
            }
        }
        _ => unreachable!(),
    };
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert() -> Result<()> {
        open_test_db().await?;
        let mut jack = GenericObject::new("person")
            .await?
            .add_string_attribute("name", "Jack")
            .await?;
        jack.insert().await?;
        let mut fred = GenericObject::new("person")
            .await?
            .add_string_attribute("name", "Fred")
            .await?
            .add_int_attribute("age", 62)
            .await?;
        fred.insert().await?;
        let objects = read_all("person").await?;
        for object in objects {
            if object.get_string_attribute("name").unwrap() == "Jack" {
                assert_eq!(jack, object);
            } else {
                assert_eq!(fred, object);
            }
        }
        Ok(())
    }
}
