use diesel::{Connection, PgConnection};

pub fn create_conn<S: AsRef<str>>(host: S, username: S, password: S, database: S) -> PgConnection {
    let url = concat_string!(
        "postgres://",
        &username,
        ":",
        &password,
        "@",
        &host,
        "/",
        &database
    );

    PgConnection::establish(&url).expect("Error connecting to postgres database!")
}
