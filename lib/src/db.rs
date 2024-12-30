use diesel::{Connection, PgConnection};

pub fn create_conn<S: AsRef<str>>(host: S, username: S, password: S, database: S) -> PgConnection {
    let url = concat_string!(
        "postgres://",
        urlencoding::encode(username.as_ref()),
        ":",
        urlencoding::encode(password.as_ref()),
        "@",
        &host,
        "/",
        &database
    );

    PgConnection::establish(&url).expect("Error connecting to postgres database!")
}
