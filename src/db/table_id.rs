use surrealdb::sql::{Id, Thing, Value};

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
