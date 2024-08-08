use crate::db::DB;
use crate::error::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Kind;

static OBJECT_TYPE_TABLE: &str = "_object_type";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObjectTypeAttribute {
    pub(crate) datatype: Kind,
    pub(crate) id: bool,
}

impl ObjectTypeAttribute {
    pub fn new(datatype: Kind, id: bool) -> Self {
        Self { datatype, id }
    }

    pub fn datatype(&self) -> &Kind {
        &self.datatype
    }

    pub fn is_id_part(&self) -> bool {
        self.id
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObjectType {
    pub(crate) type_name: String,
    pub(crate) attributes: IndexMap<String, ObjectTypeAttribute>,
    id_parts: Vec<String>,
}

impl ObjectType {
    pub fn new(type_name: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
            attributes: IndexMap::new(),
            id_parts: Vec::new(),
        }
    }

    pub fn type_name(&self) -> &String {
        &self.type_name
    }

    pub async fn get_object_type(type_name: &str) -> Result<Option<ObjectType>> {
        Ok(ObjectType::get(type_name).await?)
    }

    pub async fn get_object_type_required(type_name: &str) -> Result<ObjectType> {
        match ObjectType::get(type_name).await? {
            Some(object_type) => Ok(object_type),
            None => Err(format!("Unknown object type '{}'", type_name).into()),
        }
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    pub async fn add_attribute(
        &mut self,
        name: &str,
        datatype: Kind,
        is_id_part: bool,
    ) -> Result<()> {
        if !self.attributes.contains_key(name) {
            if is_id_part {
                self.id_parts.push(name.to_string());
            }
            self.attributes.insert(
                name.to_string(),
                ObjectTypeAttribute::new(datatype, is_id_part),
            );
            self.update().await?;
        }
        Ok(())
    }

    pub async fn remove_attribute(&mut self, attribute_name: &str) -> Result<()> {
        if self.attributes.contains_key(attribute_name) {
            let attribute = self.attributes.swap_remove(attribute_name).unwrap();
            if attribute.is_id_part() {
                // TODO: Figure out how to handle ID changing in table
                unreachable!()
            };
            self.update().await?;
        }
        Ok(())
    }

    pub fn attributes(&self) -> &IndexMap<String, ObjectTypeAttribute> {
        &self.attributes
    }

    pub async fn get_all() -> Result<Vec<ObjectType>> {
        Ok(DB.select(OBJECT_TYPE_TABLE).await?)
    }

    pub async fn get(type_name: &str) -> Result<Option<ObjectType>> {
        Ok(DB.select((OBJECT_TYPE_TABLE, type_name)).await?)
    }

    pub async fn insert(&self) -> Result<Option<ObjectType>> {
        Ok(DB
            .insert((OBJECT_TYPE_TABLE, self.type_name.clone()))
            .content(self)
            .await?)
    }

    pub async fn update(&self) -> Result<Option<ObjectType>> {
        Ok(DB
            .update((OBJECT_TYPE_TABLE, self.type_name.clone()))
            .content(self)
            .await?)
    }

    pub async fn delete(&self) -> Result<Option<Self>> {
        Ok(DB
            .delete((OBJECT_TYPE_TABLE, self.type_name.clone()))
            .await?)
    }
}
