use clap::Command;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::{CompleteEnv, Shell};

use crate::cli::commands::history_handler;
use crate::cli::commands::list_handler;
use crate::cli::commands::local_handler;
use crate::cli::commands::query_handler;
use crate::cli::commands::remote_handler;
use crate::cli::commands::status_handler;
use crate::cli::config::Cli;
use crate::cli::config::HistoryCommands;
use crate::cli::config::LocalCommands;
use crate::cli::config::MainCommands;
use crate::cli::config::QueryCommands;
use crate::cli::config::RemoteCommands;

pub mod app;
mod cli;
mod core;

fn completion_command() -> Command {
    disable_completion_help(Cli::command())
}

fn disable_completion_help(command: Command) -> Command {
    command
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .mut_subcommands(disable_completion_help)
}

#[tokio::main]
async fn main() {
    CompleteEnv::with_factory(completion_command).complete();

    let cli = Cli::parse();

    match &cli.command {
        MainCommands::Remote { command } => match command {
            Some(RemoteCommands::List {}) => remote_handler::list(),
            Some(RemoteCommands::Add {
                name,
                connection_string,
                database_type,
                is_secure,
            }) => remote_handler::add(
                name,
                connection_string.as_deref(),
                database_type.as_deref(),
                *is_secure,
            ),
            Some(RemoteCommands::Remove { name }) => remote_handler::remove(name),
            Some(RemoteCommands::Switch { name }) => remote_handler::switch(name),
            Some(RemoteCommands::Edit {}) => remote_handler::edit(),
            Some(RemoteCommands::Retrieve { name }) => remote_handler::retrieve(name),
            Some(RemoteCommands::Password { name, password }) => {
                remote_handler::password(name, password)
            }
            Some(RemoteCommands::Local { command }) => match command {
                Some(LocalCommands::Init) => local_handler::init(),
                None => {}
            },
            None => {}
        },
        MainCommands::Switch { name } => remote_handler::switch(name),
        MainCommands::List {
            output_format,
            table_name,
        } => {
            if let Err(e) = list_handler::list(output_format, table_name).await {
                eprintln!("Failed to list tables: {e}");
            }
        }
        MainCommands::Status {} => status_handler::status(),
        MainCommands::Query {
            query,
            output_format,
            command,
            id,
        } => match command {
            Some(QueryCommands::Last { output_format }) => query_handler::last(output_format).await,
            None => match id {
                Some(id) => query_handler::by_id(output_format, id).await,
                None => query_handler::query(query, output_format).await,
            },
        },
        MainCommands::History { command } => match command {
            Some(HistoryCommands::List {}) => history_handler::list(),
            Some(HistoryCommands::Last {}) => history_handler::last(),
            Some(HistoryCommands::Edit {}) => history_handler::edit(),
            None => {}
        },
        MainCommands::Generator { shell } => {
            eprintln!("Generating completion file for {shell}...");
            generate_dynamic_completion(*shell);
        }
    }
}

fn generate_dynamic_completion(shell: Shell) {
    unsafe {
        std::env::set_var("COMPLETE", shell.to_string());
    }

    let result = CompleteEnv::with_factory(completion_command)
        .completer("mid")
        .try_complete(["mid"], std::env::current_dir().ok().as_deref());

    match result {
        Ok(true) => eprintln!("Done!"),
        Ok(false) => eprintln!("Failed to activate dynamic completion generation"),
        Err(error) => eprintln!("Failed to generate completion file: {error}"),
    }
}
