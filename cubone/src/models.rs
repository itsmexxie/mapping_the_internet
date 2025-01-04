use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::Serialize;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Address {
    pub id: IpNetwork,
    pub allocation_state_id: String,
    pub allocation_state_comment: Option<String>,
    pub routed: bool,
    pub online: bool,
    pub top_rir_id: Option<String>,
    pub rir_id: Option<String>,
    pub autsys_id: Option<i64>,
    pub country: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct AddressMap {
    pub id: IpNetwork,
    pub allocation_state_id: String,
    pub updated_at: DateTime<Utc>,
}
