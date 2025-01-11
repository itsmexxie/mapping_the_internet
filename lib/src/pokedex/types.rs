use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
}

impl From<crate::db::models::Service> for Service {
    fn from(value: crate::db::models::Service) -> Self {
        Service { id: value.id }
    }
}

impl From<&crate::db::models::Service> for Service {
    fn from(value: &crate::db::models::Service) -> Self {
        Service {
            id: value.id.clone(),
        }
    }
}
