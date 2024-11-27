use std::fmt::Display;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::Services)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Service {
    pub id: i32,
    pub name: String,
    pub password: String,
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&concat_string!(
            "ID: ",
            self.id.to_string(),
            ", name: ",
            self.name
        ))
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::Services)]
pub struct NewService<'a> {
    pub name: &'a str,
    pub password: &'a str,
}
