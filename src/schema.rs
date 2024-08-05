use crate::db::DB;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Kind;

static SCHEMA_TABLE: &str = "_schema";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SchemaAttribute {
    name: String,
    datatype: Kind,
    id: bool,
}

impl SchemaAttribute {
    pub fn new(name: &str, datatype: Kind, id: bool) -> Self {
        Self {
            name: name.to_string(),
            datatype,
            id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn datatype(&self) -> &Kind {
        &self.datatype
    }

    pub fn is_id_part(&self) -> bool {
        self.id
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Schema {
    type_name: String,
    attributes: Vec<SchemaAttribute>,
    id_parts: Vec<String>,
}

impl Schema {
    pub fn new(type_name: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
            attributes: Vec::new(),
            id_parts: Vec::new(),
        }
    }

    pub async fn get_schema(type_name: &str) -> Result<Option<Schema>> {
        Ok(Schema::get(type_name).await?)
    }

    pub async fn get_schema_required(type_name: &str) -> Result<Schema> {
        match Schema::get(type_name).await? {
            Some(schema) => Ok(schema),
            None => Err(format!("Unknown object type '{}'", type_name).into()),
        }
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().find(|a| a.name() == name).is_some()
    }

    pub async fn add_attribute(&mut self, attribute: SchemaAttribute) -> Result<()> {
        if !self.attributes.contains(&attribute) {
            if attribute.is_id_part() {
                self.id_parts.push(attribute.name().to_string());
            }
            self.attributes.push(attribute);
            self.update().await?;
        }
        Ok(())
    }

    pub async fn remove_attribute(&mut self, attribute_name: &str) -> Result<()> {
        if let Some(index) = self
            .attributes
            .iter()
            .position(|a| a.name() == attribute_name)
        {
            let attribute = self.attributes.remove(index);
            if attribute.is_id_part() {
                // TODO: Figure out how to handle ID changing in table
                unreachable!()
            };
            self.update().await?;
        }
        Ok(())
    }

    pub fn attributes(&self) -> &Vec<SchemaAttribute> {
        &self.attributes
    }

    pub async fn get(type_name: &str) -> Result<Option<Schema>> {
        Ok(DB.select((SCHEMA_TABLE, type_name)).await?)
    }

    pub async fn insert(&self) -> Result<Option<Schema>> {
        Ok(DB
            .insert((SCHEMA_TABLE, self.type_name.clone()))
            .content(self)
            .await?)
    }

    pub async fn update(&self) -> Result<Option<Schema>> {
        Ok(DB
            .update((SCHEMA_TABLE, self.type_name.clone()))
            .content(self)
            .await?)
    }

    pub async fn delete(&self) -> Result<Option<Self>> {
        Ok(DB.delete((SCHEMA_TABLE, self.type_name.clone())).await?)
    }
}
