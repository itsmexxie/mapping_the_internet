use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub max_one: bool,
}

impl From<crate::db::models::Service> for Service {
    fn from(value: crate::db::models::Service) -> Self {
        Service {
            id: value.id,
            max_one: value.max_one,
        }
    }
}

impl From<&crate::db::models::Service> for Service {
    fn from(value: &crate::db::models::Service) -> Self {
        Service {
            id: value.id.clone(),
            max_one: value.max_one,
        }
    }
}
