use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ValueResponse<T> {
    pub value: T,
}
