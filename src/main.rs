use clap::Parser;

use crate::cli::{
    Cli,
    commands::{
        Commands, history::handle_history_command, list::handle_list_command,
        query::handle_query_command, remote::handle_remote_command, status::handle_status_command,
    },
};

use crate::core::query::QueryOutputFormat;

mod cli;
mod core;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }

        Some(Commands::Remote { command }) => handle_remote_command(command),
        Some(Commands::List { output_format }) => {
            handle_list_command(
                output_format
                    .as_ref()
                    .unwrap_or(&QueryOutputFormat::Table)
                    .clone(),
            )
            .await
        }
        Some(Commands::Status {}) => handle_status_command(),
        Some(Commands::Query {
            query,
            output_format,
            command,
        }) => handle_query_command(command, query, output_format).await,
        Some(Commands::History { command }) => handle_history_command(command).await,
        _ => {}
    }
}
