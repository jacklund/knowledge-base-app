use crate::db::DB;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, RwLock};
use surrealdb::Result;

pub static SCHEMAS: LazyLock<Mutex<Schemas>> = LazyLock::new(|| Mutex::new(Schemas::new()));

static SCHEMA_TABLE: &str = "_schema";

pub struct Schemas {
    map: HashMap<String, RwLock<Schema>>,
}

impl Schemas {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub async fn get(&mut self, type_name: &str) -> surrealdb::Result<&RwLock<Schema>> {
        if !self.map.contains_key(type_name) {
            let schema = match Schema::get(type_name).await? {
                Some(schema) => schema,
                None => {
                    let schema = Schema::new(type_name);
                    schema.insert().await?;
                    schema
                }
            };
            self.map.insert(type_name.to_string(), RwLock::new(schema));
        }
        Ok(self.map.get(type_name).unwrap())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SchemaAttribute {
    name: String,
    id: bool,
}

impl SchemaAttribute {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: false,
        }
    }

    fn in_id(&mut self) {
        self.id = true;
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub async fn add_schema_attribute(type_name: &str, attribute: &str, id: bool) -> Result<()> {
    let mut schema_attribute = SchemaAttribute::new(attribute);
    if id {
        schema_attribute.in_id();
    }
    let mut schemas = SCHEMAS.lock().unwrap();
    let mut schema = schemas.get(type_name).await?.write().unwrap();
    schema.add_attribute(schema_attribute).await?;

    Ok(())
}

pub async fn has_attribute(type_name: &str, attribute: &str) -> Result<bool> {
    let mut schemas = SCHEMAS.lock().unwrap();
    let schema = schemas.get(type_name).await?.read().unwrap();
    Ok(schema.attributes.iter().find(|a| a.name() == attribute) != None)
}

pub async fn remove_schema_attribute(type_name: &str, attribute: &str) -> Result<()> {
    let mut schemas = SCHEMAS.lock().unwrap();
    let mut schema = schemas.get(type_name).await?.write().unwrap();
    schema.remove_attribute(attribute).await?;

    Ok(())
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Schema {
    type_name: String,
    attributes: Vec<SchemaAttribute>,
}

impl Schema {
    pub fn new(type_name: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
            attributes: Vec::new(),
        }
    }

    async fn add_attribute(&mut self, attribute: SchemaAttribute) -> Result<()> {
        if !self.attributes.contains(&attribute) {
            self.attributes.push(attribute);
            self.update().await?;
        }
        Ok(())
    }

    async fn remove_attribute(&mut self, attribute_name: &str) -> Result<()> {
        if let Some(index) = self
            .attributes
            .iter()
            .position(|a| a.name() == attribute_name)
        {
            self.attributes.remove(index);
            self.update().await?;
        }
        Ok(())
    }

    pub fn attributes(&self) -> &Vec<SchemaAttribute> {
        &self.attributes
    }

    pub async fn get(type_name: &str) -> surrealdb::Result<Option<Schema>> {
        DB.select((SCHEMA_TABLE, type_name)).await
    }

    pub async fn insert(&self) -> surrealdb::Result<Option<Schema>> {
        DB.insert((SCHEMA_TABLE, self.type_name.clone()))
            .content(self)
            .await
    }

    pub async fn update(&self) -> surrealdb::Result<Option<Schema>> {
        DB.update((SCHEMA_TABLE, self.type_name.clone()))
            .content(self)
            .await
    }
}
