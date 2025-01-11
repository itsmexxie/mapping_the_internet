use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
