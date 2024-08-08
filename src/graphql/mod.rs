use crate::object_type;
use async_graphql::*;
use surrealdb::sql::Kind;

#[derive(Clone, Copy, Eq, Enum, PartialEq)]
enum DataType {
    Bool,
    Int,
    Float,
    String,
}

impl From<&Kind> for DataType {
    fn from(kind: &Kind) -> Self {
        match kind {
            Kind::Bool => DataType::Bool,
            Kind::Int => DataType::Int,
            Kind::Float => DataType::Float,
            Kind::String => DataType::String,
            _ => unreachable!(),
        }
    }
}

impl From<&DataType> for Kind {
    fn from(datatype: &DataType) -> Self {
        match datatype {
            DataType::Bool => Kind::Bool,
            DataType::Int => Kind::Int,
            DataType::Float => Kind::Float,
            DataType::String => Kind::String,
        }
    }
}

#[derive(SimpleObject, InputObject)]
#[graphql(name = "ObjectTypeField", input_name = "ObjectTypeFieldInput")]
pub struct ObjectTypeField {
    name: String,
    datatype: DataType,
    id: bool,
}

#[derive(SimpleObject)]
pub struct ObjectType {
    name: String,
    fields: Vec<ObjectTypeField>,
}

impl From<&object_type::ObjectType> for ObjectType {
    fn from(o: &object_type::ObjectType) -> Self {
        Self {
            name: o.type_name.clone(),
            fields: o
                .attributes
                .iter()
                .map(|(k, a)| ObjectTypeField {
                    name: k.clone(),
                    datatype: a.datatype().into(),
                    id: a.is_id_part(),
                })
                .collect(),
        }
    }
}

impl From<object_type::ObjectType> for ObjectType {
    fn from(o: object_type::ObjectType) -> Self {
        Self {
            name: o.type_name.clone(),
            fields: o
                .attributes
                .iter()
                .map(|(k, a)| ObjectTypeField {
                    name: k.clone(),
                    datatype: a.datatype().into(),
                    id: a.is_id_part(),
                })
                .collect(),
        }
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn get_object_types(&self) -> Result<Vec<ObjectType>> {
        Ok(object_type::ObjectType::get_all()
            .await?
            .iter()
            .map(|ot| ot.into())
            .collect())
    }

    async fn get_object_type(&self, name: String) -> Result<Option<ObjectType>> {
        Ok(object_type::ObjectType::get_object_type(&name)
            .await?
            .map(|o| o.into()))
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn add_object_type<'a>(
        &self,
        ctx: &Context<'a>,
        name: String,
        fields: Vec<ObjectTypeField>,
    ) -> Result<ObjectType> {
        let mut object_type = object_type::ObjectType::new(&name);
        for field in fields {
            object_type
                .add_attribute(
                    &field.name(ctx).await?,
                    field.datatype(ctx).await?.into(),
                    *field.id(ctx).await?,
                )
                .await?;
        }
        object_type.insert().await.unwrap();
        Ok((&object_type).into())
    }
}
