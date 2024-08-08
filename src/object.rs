use crate::db::DB;
use crate::error::Result;
use crate::object_type::ObjectType;
use crate::tag::Tag;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Object {
    type_name: String,
    attributes: IndexMap<String, String>,
    tags: Vec<Tag>,
}

impl Object {
    pub async fn new(type_name: &str) -> Result<Object> {
        let object = Self {
            type_name: type_name.to_string(),
            attributes: IndexMap::new(),
            tags: Vec::new(),
        };
        ObjectType::get_object_type_required(type_name).await?;
        Ok(object)
    }

    pub async fn add_attribute(mut self, name: &str, value: &str) -> Result<Object> {
        let object_type = ObjectType::get_object_type_required(&self.type_name).await?;

        if !object_type.has_attribute(name) {
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

    pub async fn insert(&mut self, object_type: Option<ObjectType>) -> Result<()> {
        let object_type = match object_type {
            Some(object_type) => object_type,
            None => ObjectType::get_object_type_required(&self.type_name).await?,
        };
        // TODO: Add ID to insert
        let id_parts: Vec<String> = object_type
            .attributes()
            .iter()
            .filter_map(|(k, a)| {
                if a.is_id_part() {
                    Some(k.to_string())
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
        let mut fields = object_type
            .attributes()
            .iter()
            .filter_map(|(k, _a)| match self.attributes.get(k) {
                Some(_) => Some(k.to_string()),
                None => None,
            })
            .collect::<VecDeque<String>>();
        fields.push_front(String::from("id"));
        let field_names: String = fields.iter().join(", ");
        let mut values = object_type
            .attributes()
            .iter()
            .filter_map(|(k, a)| match self.attributes.get(k) {
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
