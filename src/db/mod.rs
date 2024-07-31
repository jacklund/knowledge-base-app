use crate::db::generic_object::{GenericObject, TableId};
use std::sync::LazyLock;
use surrealdb::engine::local::{Db, Mem, RocksDb};
use surrealdb::sql::{Array, Object as DbObject, Value};
use surrealdb::{Result, Surreal};

pub mod generic_object;

pub static DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());

// Open the DB
pub async fn open_db() -> Result<()> {
    // let mut db_dir = data_local_dir().unwrap();
    // db_dir.push("knowledge_repo");
    // DB.connect::<RocksDb>(db_dir).await
    DB.connect::<Mem>(()).await?;
    DB.use_ns("test").use_db("test").await?;
    Ok(())
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

pub fn parse_array(array: Array) -> Vec<DbObject> {
    array.iter().map(|v| parse_value(v.clone())).collect()
}

pub async fn read_all(table: &str) -> Result<Vec<GenericObject>> {
    let mut ret = Vec::new();
    let mut response = DB.query(format!("select * from {table}")).await?;
    let result: Value = response.take(0)?;
    match result {
        Value::Array(array) => {
            for obj in parse_array(array).iter_mut() {
                let id: TableId = obj.remove("id").unwrap().into();
                let mut object = GenericObject::new(&id.table_name());
                object.set_table_id(id);
                for (key, value) in obj.iter() {
                    object = object.add_kv_attribute(key, value.clone().into());
                }
                ret.push(object);
            }
        }
        _ => unreachable!(),
    };
    Ok(ret)
}
