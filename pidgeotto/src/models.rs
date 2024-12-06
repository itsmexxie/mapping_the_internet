use diesel::prelude::*;
use mtilib::types::Rir;

// #[derive(Queryable, Selectable, Debug)]
// #[diesel(table_name = crate::schema::Services)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct Service {
//     pub id: i32,
//     pub name: String,
//     pub password: String,
//     pub max_one: bool,
// }

#[derive(Insertable, Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::Addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Address {
    pub id: ipnetwork::IpNetwork,
    pub allocation_state_id: String,
    pub allocation_state_comment: Option<String>,
    pub top_rir_id: Option<Rir>,
    pub rir_id: Option<Rir>,
    pub asn_id: Option<i32>,
    pub country: Option<String>,
    pub routed: bool,
    pub online: bool,
    pub ports: Vec<i32>,
}

#[derive(Debug, PartialEq, Hash, Eq, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::Asns)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Asn {
    pub id: i32,
}
