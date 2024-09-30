use anyhow::anyhow;
use leptos::*;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    EnumIter,
    EnumString,
    IntoStaticStr,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
)]
pub(crate) enum DataType {
    #[default]
    Bool,
    Int,
    Float,
    String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ObjectTypeAttribute {
    name: String,
    data_type: DataType,
    is_id_part: bool,
}

impl ObjectTypeAttribute {
    pub fn new(name: &str, data_type: DataType, is_id_part: bool) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            is_id_part,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    pub fn is_id_part(&self) -> bool {
        self.is_id_part
    }
}

impl IntoView for ObjectTypeAttribute {
    fn into_view(self) -> View {
        view! {
            {self.name}
            :
            {<DataType as Into<&'static str>>::into(self.data_type)}
        }
        .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ObjectType {
    name: String,
    attributes: Vec<ObjectTypeAttribute>,
    id_parts: Vec<String>,
}

impl ObjectType {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            attributes: Vec::new(),
            id_parts: Vec::new(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    fn get_attribute(&self, name: &str) -> Option<&ObjectTypeAttribute> {
        self.attributes.iter().find(|a| a.name == name)
    }

    fn has_attribute(&self, name: &str) -> bool {
        self.get_attribute(name).is_none()
    }

    pub fn add_attribute(
        &mut self,
        name: &str,
        data_type: DataType,
        is_id_part: bool,
    ) -> anyhow::Result<()> {
        if !self.has_attribute(name) {
            self.attributes
                .push(ObjectTypeAttribute::new(name, data_type, is_id_part));
            Ok(())
        } else {
            Err(anyhow!("Attribute named {} already exists", name))
        }
    }
}

impl IntoView for ObjectType {
    fn into_view(self) -> View {
        view! {
            <p>{self.name}</p>
            <p>{self.attributes.iter().map(|a| a.clone().into_view()).collect::<Vec<View>>()}</p>
        }
        .into()
    }
}
