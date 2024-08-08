use crate::error::Result;
use crate::object::Object;
use crate::object_type::ObjectType;
use std::sync::LazyLock;
use surrealdb::engine::any::Any;
use surrealdb::engine::local::{Db, Mem, RocksDb};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::sql::{Array, Object as DbObject, Value};
use surrealdb::Surreal;

// pub static DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());
pub static DB: LazyLock<Surreal<Any>> = LazyLock::new(|| Surreal::init());

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
    // DB.connect::<Mem>(()).await?;
    DB.connect("http://localhost:8000").await?;
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
pub async fn read_all(table: &str) -> Result<Vec<Object>> {
    // Get a copy of the type for the object
    let object_type = ObjectType::get_object_type_required(table).await?;

    // Since we're reading a generic object, we need to parse
    // it into the object ourselves
    let mut ret = Vec::new();
    let mut response = DB.query(format!("select * from {table}")).await?;

    // Read as array of Value
    let result: Value = response.take(0)?;
    match result {
        Value::Array(array) => {
            for obj in parse_array(array).iter_mut() {
                let mut object = Object::new(table).await?;

                // Load the attributes in the same order as in the object_type
                for (name, _attr) in object_type.attributes() {
                    if let Some((key, value)) = obj.iter().find(|(k, _)| *k == name) {
                        object = object
                            .add_attribute(key, &value.clone().as_raw_string())
                            .await?;
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
    use crate::error::Result;
    use crate::object_type::ObjectType;
    use surrealdb::sql::Kind;

    #[tokio::test]
    async fn test_insert() -> Result<()> {
        open_test_db().await?;
        let mut object_type = ObjectType::new("person");
        object_type
            .add_attribute("name", Kind::String, true)
            .await?;
        object_type.add_attribute("age", Kind::Int, false).await?;
        let mut jack = Object::new("person")
            .await?
            .add_attribute("name", "Jack")
            .await?;
        jack.insert(Some(object_type.clone())).await?;
        let mut fred = Object::new("person")
            .await?
            .add_attribute("name", "Fred")
            .await?
            .add_attribute("age", "62")
            .await?;
        fred.insert(Some(object_type)).await?;
        let objects = read_all("person").await?;
        for object in objects {
            if object.get_attribute("name").unwrap() == "Jack" {
                assert_eq!(jack, object);
            } else {
                assert_eq!(fred, object);
            }
        }
        Ok(())
    }
}
