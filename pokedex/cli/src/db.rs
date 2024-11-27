use config::Config;
use diesel::{Connection, PgConnection};

pub fn create_conn(config: Config) -> PgConnection {
    let db_host: String = config
        .get("database.host")
        .expect("database.host must be set!");
    let db_username = config
        .get_string("database.username")
        .expect("database.username must be set!");
    let db_password = config
        .get_string("database.password")
        .expect("database.password must be set!");
    let db_db = config
        .get_string("database.database")
        .expect("database.database must be set!");
    let db_url = concat_string!(
        "postgres://",
        &db_username,
        ":",
        &db_password,
        "@",
        &db_host,
        "/",
        &db_db
    );

    PgConnection::establish(&db_url).expect("Error connecting to postgres database!")
}
