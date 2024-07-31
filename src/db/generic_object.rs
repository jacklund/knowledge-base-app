use crate::db::DB;
use chrono::{DateTime, Utc};
use std::fmt;
use std::time::Duration;
use surrealdb::sql::{Id, Number, Thing, Value};
use surrealdb::Result;

#[derive(Debug)]
pub enum AttributeValue {
    // Work around warnings due to Debug derive above
    #[allow(dead_code)]
    Bool(bool),

    #[allow(dead_code)]
    Int(i64),

    #[allow(dead_code)]
    Float(f64),

    #[allow(dead_code)]
    String(String),

    #[allow(dead_code)]
    Duration(Duration),

    #[allow(dead_code)]
    DateTime(DateTime<Utc>),

    #[allow(dead_code)]
    Array(Vec<AttributeValue>),

    #[allow(dead_code)]
    Bytes(Vec<u8>),
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", b),
            Self::Int(i) => write!(f, "{}", i),
            Self::Float(float) => write!(f, "{}", float),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::DateTime(d) => write!(f, "{}", d),
            _ => unreachable!(),
        }
    }
}

impl From<Value> for AttributeValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(b) => Self::Bool(b),
            Value::Number(n) => match n {
                Number::Int(num) => Self::Int(num),
                Number::Float(num) => Self::Float(num),
                Number::Decimal(_) => unreachable!(),
            },
            Value::Strand(string) => Self::String(string.to_string()),
            Value::Duration(dur) => Self::Duration(*dur),
            Value::Datetime(dt) => Self::DateTime(*dt),
            Value::Array(array) => Self::Array(array.iter().map(|v| v.clone().into()).collect()),
            Value::Bytes(bytes) => Self::Bytes((*bytes).clone()),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Attribute {
    key: String,
    value: AttributeValue,
}

impl Attribute {
    pub fn new(key: String, value: AttributeValue) -> Self {
        Self { key, value }
    }

    fn to_db_string(&self) -> String {
        format!("{}: {}", self.key, self.value)
    }
}

#[derive(Debug, Default)]
struct AttributeList {
    list: Vec<Attribute>,
}

impl AttributeList {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn add(&mut self, attribute: Attribute) {
        self.list.push(attribute);
    }

    fn to_db_string(&self) -> String {
        let mut db_string = "[{".to_string();
        db_string.push_str(
            &self
                .list
                .iter()
                .map(|a| a.to_db_string())
                .collect::<Vec<String>>()
                .join(", "),
        );
        db_string.push_str("}]");
        db_string
    }
}

#[derive(Debug)]
pub struct TableId {
    table_name: String,
    id: Option<String>,
}

impl TableId {
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            id: None,
        }
    }

    pub fn table_name(&self) -> String {
        self.table_name.clone()
    }

    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }
}

impl From<Value> for TableId {
    fn from(item: Value) -> Self {
        match item {
            Value::Thing(thing) => thing.into(),
            _ => unreachable!(),
        }
    }
}

impl From<Thing> for TableId {
    fn from(thing: Thing) -> Self {
        Self {
            table_name: thing.tb,
            id: match thing.id {
                Id::String(id) => Some(id),
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Debug)]
pub struct GenericObject {
    id: TableId,
    attributes: AttributeList,
}

impl GenericObject {
    pub fn new(type_name: &str) -> GenericObject {
        Self {
            id: TableId::new(type_name),
            attributes: AttributeList::default(),
        }
    }

    pub fn set_table_id(&mut self, id: TableId) {
        self.id = id;
    }

    pub fn table_name(&self) -> String {
        self.id.table_name()
    }

    fn add_attribute(mut self, attribute: Attribute) -> GenericObject {
        // TODO: Check if attribute exists
        self.attributes.add(attribute);
        self
    }

    pub fn add_kv_attribute(self, key: &str, value: AttributeValue) -> GenericObject {
        self.add_attribute(Attribute::new(key.to_string(), value))
    }

    pub fn add_string_attribute(self, key: &str, value: &str) -> GenericObject {
        self.add_kv_attribute(key, AttributeValue::String(value.to_string()))
    }

    pub fn add_int_attribute(self, key: &str, value: i64) -> GenericObject {
        self.add_kv_attribute(key, AttributeValue::Int(value))
    }

    pub async fn insert(&mut self) -> Result<()> {
        // TODO: Add ID to insert
        let mut response = DB
            .query(format!(
                "insert into {table_name} {attributes}",
                table_name = self.table_name(),
                attributes = self.attributes.to_db_string(),
            ))
            .await?;
        self.set_table_id(response.take::<Value>(0)?.first().record().unwrap().into());
        Ok(())
    }
}
