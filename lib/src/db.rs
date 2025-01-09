use concat_string::concat_string;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use urlencoding;

pub mod models;

pub type DbPool = Arc<Pool<Postgres>>;

pub fn url<S: AsRef<str>>(hostname: S, username: S, password: S, database: S) -> String {
    concat_string!(
        "postgres://",
        urlencoding::encode(username.as_ref()),
        ":",
        urlencoding::encode(password.as_ref()),
        "@",
        &hostname,
        "/",
        &database
    )
}
