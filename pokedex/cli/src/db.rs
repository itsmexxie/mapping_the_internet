use sqlx::{Connection, PgConnection};

use crate::settings::Settings;

pub async fn connect(settings: &Settings) -> PgConnection {
    PgConnection::connect(&mtilib::db::url(
        &settings.database.hostname,
        &settings.database.username,
        &settings.database.password,
        &settings.database.database,
    ))
    .await
    .expect("Failed to connect to the database!")
}
