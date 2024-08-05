use crate::db::DB;
use serde::{Deserialize, Serialize};
use std::fmt;
use surrealdb::Result;

const TAG_TABLE: &str = "_tag";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tag {
    name: String,
}

impl Tag {
    pub async fn get(name: &str) -> Result<Option<Self>> {
        DB.select((TAG_TABLE, name)).await
    }

    pub async fn new(name: &str) -> Result<Option<Self>> {
        let tag = Self {
            name: name.to_string(),
        };
        DB.create((TAG_TABLE, name)).content(tag).await
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.name)
    }
}
