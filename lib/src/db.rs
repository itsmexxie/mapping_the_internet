use concat_string::concat_string;
#[cfg(feature = "diesel")]
use diesel::{Connection, PgConnection};
#[cfg(feature = "sqlx")]
use sqlx::{Pool, Postgres};
#[cfg(feature = "sqlx")]
use std::sync::Arc;
use urlencoding;

#[cfg(feature = "sqlx")]
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

#[cfg(feature = "diesel")]
pub fn create_conn<S: AsRef<str>>(
    hostname: S,
    username: S,
    password: S,
    database: S,
) -> PgConnection {
    PgConnection::establish(&url(hostname, username, password, database))
        .expect("Error connecting to postgres database!")
}
