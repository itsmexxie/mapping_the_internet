use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::Services)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Service {
    pub id: i32,
    pub name: String,
    pub password: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::Services)]
pub struct NewService<'a> {
    pub name: &'a str,
    pub password: &'a str,
}
