use crate::schema::test;
use diesel::prelude::*;

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = test)]
pub struct Test {
    pub id: Option<i32>,
    pub name: String,
}

impl Test {
    pub fn new(name: &str) -> Self {
        Self {
            id: None,
            name: name.to_string(),
        }
    }
}
