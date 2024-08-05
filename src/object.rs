use crate::db::DB;
use crate::error::Result;
use crate::schema::Schema;
use crate::tag::Tag;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use surrealdb::sql::{Kind, Value};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Object {
    type_name: String,
    attributes: HashMap<String, String>,
    tags: Vec<Tag>,
}

impl Object {
    pub async fn new(type_name: &str) -> Result<Object> {
        let object = Self {
            type_name: type_name.to_string(),
            attributes: HashMap::new(),
            tags: Vec::new(),
        };
        Schema::get_schema_required(type_name).await?;
        Ok(object)
    }

    pub async fn add_attribute(mut self, name: &str, value: &str) -> Result<Object> {
        let schema = Schema::get_schema_required(&self.type_name).await?;

        if !schema.has_attribute(name) {
            return Err(
                format!("Unknown attribute '{}' for type '{}'", name, self.type_name).into(),
            );
        }
        self.attributes.insert(name.to_string(), value.to_string());
        Ok(self)
    }

    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    pub async fn insert(&mut self, schema: Option<Schema>) -> Result<()> {
        let schema = match schema {
            Some(schema) => schema,
            None => Schema::get_schema_required(&self.type_name).await?,
        };
        // TODO: Add ID to insert
        let id_parts: Vec<String> = schema
            .attributes()
            .iter()
            .filter_map(|a| {
                if a.is_id_part() {
                    Some(a.name().to_string())
                } else {
                    None
                }
            })
            .collect();
        let mut parts = vec![];
        for part in id_parts {
            match self.attributes.get(&part) {
                Some(attr) => parts.push(attr.clone()),
                None => return Err("Don't have complete ID for database".into()),
            }
        }
        let id = parts.join(":");
        let mut fields = schema
            .attributes()
            .iter()
            .filter_map(|a| match self.attributes.get(a.name()) {
                Some(_) => Some(a.name().to_string()),
                None => None,
            })
            .collect::<VecDeque<String>>();
        fields.push_front(String::from("id"));
        let field_names: String = fields.iter().join(", ");
        let mut values = schema
            .attributes()
            .iter()
            .filter_map(|a| match self.attributes.get(a.name()) {
                Some(value) => Some((a.datatype().clone(), value.clone())),
                None => None,
            })
            .map(|(kind, value)| format!("<{}> \"{}\"", kind, value))
            .collect::<VecDeque<String>>();
        values.push_front(format!("\"{}\"", id));
        let field_values = values.iter().join(", ");
        DB.query(format!(
            "insert into {} ({}) values ({})",
            self.type_name, field_names, field_values,
        ))
        .await?;
        Ok(())
    }
}
