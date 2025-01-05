use core::panic;

use clap::Parser;
use cli::ServiceCommands;
use mtilib::db::models::Service;
use settings::Settings;

pub mod cli;
pub mod db;
pub mod settings;

use crate::cli::Commands;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Settings
    let (_, settings): (_, Settings) = mtilib::settings::deserialize_from_config("cli.config.toml");

    // Clap
    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Services(services_args) => {
            let service_cmd = services_args.command.unwrap_or(ServiceCommands::List);
            match service_cmd {
                ServiceCommands::List => {
                    match sqlx::query_as::<_, Service>(
                        r#"
						SELECT *
						FROM "Services"
						"#,
                    )
                    .fetch_all(&mut db::connect(&settings).await)
                    .await
                    {
                        Ok(rows) => {
                            for row in rows {
                                println!("{:?}", row);
                            }
                        }
                        Err(error) => panic!("Error while fetching services: {}", error),
                    }
                }
                ServiceCommands::Create {
                    name,
                    password,
                    max_one,
                } => {
                    let hashed_password = bcrypt::hash(&password, 12).unwrap();

                    sqlx::query(
                        r#"
						INSERT INTO "Services"
						VALUES ($1, $2, $3)
						"#,
                    )
                    .bind(name)
                    .bind(hashed_password)
                    .bind(max_one)
                    .execute(&mut db::connect(&settings).await)
                    .await
                    .expect("Failed to create the new service!");
                }
                ServiceCommands::Delete { id } => {
                    if let Err(error) = sqlx::query_as::<_, Service>(
                        r#"
						DELETE FROM "Services"
						WHERE id = $1
						"#,
                    )
                    .bind(id)
                    .fetch_all(&mut db::connect(&settings).await)
                    .await
                    {
                        panic!("Failed to delete the specified service! ({})", error)
                    }
                }
            }
        }
    }
}
