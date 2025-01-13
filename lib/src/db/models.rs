use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug, sqlx::FromRow)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AddressMap {
    pub id: IpNetwork,
    pub allocation_state_id: String,
    pub routed: bool,
    pub online: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Service {
    pub id: String,
    pub password: String,
}

#[derive(Debug, sqlx::FromRow)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ServiceUnit {
    pub id: Uuid,
    pub service_id: String,
    pub address: Option<String>,
    pub port: Option<i32>,
    pub created_at: DateTime<Utc>,
}
