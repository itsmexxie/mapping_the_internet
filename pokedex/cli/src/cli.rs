use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Services(ServicesArgs),
}

#[derive(Args)]
pub struct ServicesArgs {
    #[command(subcommand)]
    pub command: Option<ServiceCommands>,
}

#[derive(Subcommand)]
pub enum ServiceCommands {
    List,
    Create { name: String, password: String },
    Delete { id: i32 },
}
