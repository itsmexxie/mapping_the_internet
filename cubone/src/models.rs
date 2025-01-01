use diesel::prelude::*;
use mtilib::types::Rir;
use serde::Serialize;

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::Addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Address {
    pub id: ipnetwork::IpNetwork,
    #[serde(rename(serialize = "allocation_state"))]
    pub allocation_state_id: String,
    pub allocation_state_comment: Option<String>,
    pub routed: bool,
    pub online: bool,
    #[serde(rename(serialize = "top_rir"))]
    pub top_rir_id: Option<Rir>,
    #[serde(rename(serialize = "rir"))]
    pub rir_id: Option<Rir>,
    #[serde(rename(serialize = "autsys"))]
    pub autsys_id: Option<i32>,
    pub country: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, PartialEq, Hash, Eq, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::Autsyses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Autsys {
    pub id: i32,
}
