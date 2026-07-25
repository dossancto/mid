use clap::Subcommand;

use crate::{
    cli::commands::{history::HistoryCommands, query::QueryCommands, remote::RemoteCommands},
    core::query::QueryOutputFormat,
};

pub mod history;
pub mod list;
pub mod query;
pub mod remote;
pub mod status;

#[derive(Subcommand)]
pub enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },

    Remote {
        #[command(subcommand)]
        command: Option<RemoteCommands>,
    },

    Status {},

    List {
        // #[command(subcommand)]
        // command: Option<list::ListCommands>,
        #[arg(short, long)]
        output_format: Option<QueryOutputFormat>,
    },

    #[command(args_conflicts_with_subcommands = true)]
    Query {
        #[arg()]
        query: Option<String>,
        #[arg(short, long)]
        output_format: Option<QueryOutputFormat>,
        #[command(subcommand)]
        command: Option<QueryCommands>,
    },

    History {
        #[command(subcommand)]
        command: Option<HistoryCommands>,
    },
}
