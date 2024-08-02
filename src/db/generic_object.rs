use crate::db::DB;
use crate::schema::add_schema_attribute;
use chrono::{DateTime, Utc};
use std::fmt;
use std::time::Duration;
use surrealdb::sql::{Id, Number, Thing, Value};
use surrealdb::Result;

#[derive(Clone, Debug, PartialEq)]
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
            Value::Strand(strand) => Self::String(strand.as_string()),
            Value::Duration(dur) => Self::Duration(*dur),
            Value::Datetime(dt) => Self::DateTime(*dt),
            Value::Array(array) => Self::Array(array.iter().map(|v| v.clone().into()).collect()),
            Value::Bytes(bytes) => Self::Bytes((*bytes).clone()),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Debug, Default, PartialEq)]
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

    pub fn find(&self, key: &str) -> Option<AttributeValue> {
        self.list
            .iter()
            .find(|&a| a.key == key)
            .map(|a| a.value.clone())
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

#[derive(Debug, PartialEq)]
pub struct TableId {
    table_name: String,
    id: Option<Id>,
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

    pub fn id(&self) -> Option<Id> {
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
            id: Some(thing.id),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GenericObject {
    id: TableId,
    attributes: AttributeList,
}

impl GenericObject {
    pub async fn new(type_name: &str) -> Result<GenericObject> {
        Ok(Self {
            id: TableId::new(type_name),
            attributes: AttributeList::default(),
        })
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

    pub async fn add_kv_attribute(self, key: &str, value: AttributeValue) -> Result<GenericObject> {
        add_schema_attribute(&self.id.table_name(), key, false).await?;
        Ok(self.add_attribute(Attribute::new(key.to_string(), value)))
    }

    pub async fn add_string_attribute(self, key: &str, value: &str) -> Result<GenericObject> {
        self.add_kv_attribute(key, AttributeValue::String(value.to_string()))
            .await
    }

    pub async fn add_int_attribute(self, key: &str, value: i64) -> Result<GenericObject> {
        self.add_kv_attribute(key, AttributeValue::Int(value)).await
    }

    pub fn get_string_attribute(&self, key: &str) -> Option<String> {
        let value = self.attributes.find(key);
        if value.is_some() {
            if let AttributeValue::String(string) = value.unwrap() {
                Some(string)
            } else {
                None
            }
        } else {
            None
        }
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
