use std::fmt::Display;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::Services)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Service {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub max_one: bool,
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

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::ServiceUnits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServiceUnit {
    pub id: String,
    pub service_id: i32,
    pub address: Option<String>,
    pub port: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ServiceUnits)]
pub struct NewServiceUnit<'a> {
    pub id: &'a str,
    pub service_id: i32,
    pub address: Option<String>,
    pub port: Option<i32>,
}
