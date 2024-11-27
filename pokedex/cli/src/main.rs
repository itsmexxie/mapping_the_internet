use clap::Parser;
use cli::ServiceCommands;
use config::Config;
use diesel::prelude::*;
use tracing::debug;

#[macro_use(concat_string)]
extern crate concat_string;

pub mod cli;
pub mod db;
pub mod models;
pub mod schema;

use crate::cli::Commands;
use crate::models::{NewService, Service};
use crate::schema::Services;

fn main() {
    // Logging
    tracing_subscriber::fmt::init();

    // Config
    let config = Config::builder()
        .add_source(config::File::with_name("cli.config.toml"))
        .build()
        .unwrap();

    // Clap
    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Services(services_args) => {
            let service_cmd = services_args.command.unwrap_or(ServiceCommands::List);
            match service_cmd {
                ServiceCommands::List => {
                    let pg_conn = &mut db::create_conn(config);

                    let results = Services::dsl::Services
                        .select(Service::as_select())
                        .load(pg_conn)
                        .unwrap();

                    for result in results {
                        println!("{}", result);
                    }
                }
                ServiceCommands::Create {
                    name: service_name,
                    password: service_password,
                } => {
                    let pg_conn = &mut db::create_conn(config);

                    let hashed_password = bcrypt::hash(&service_password, 12).unwrap();
                    let new_service = NewService {
                        name: &service_name,
                        password: &hashed_password,
                    };

                    debug!(service_password);
                    debug!(hashed_password);

                    diesel::insert_into(Services::table)
                        .values(&new_service)
                        .returning(Service::as_returning())
                        .get_result(pg_conn)
                        .expect("Error creating a new service");
                }
                ServiceCommands::Delete { id: service_id } => {
                    let pg_conn = &mut db::create_conn(config);

                    diesel::delete(Services::dsl::Services.filter(Services::id.eq(service_id)))
                        .execute(pg_conn)
                        .expect("Error while deleting a service");
                }
            }
        }
    }
}
