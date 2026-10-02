use clap::Parser;
use clap::Subcommand;
use clap_complete::ArgValueCompleter;
use clap_complete::Shell;

use crate::core::config::completer::{complete_remotes, complete_tables};
use crate::core::query::QueryOutputFormat;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = None,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: MainCommands,
}

#[derive(Subcommand)]
pub enum MainCommands {
    #[command(arg_required_else_help = true)]
    Remote {
        #[command(subcommand)]
        command: Option<RemoteCommands>,
    },

    Switch {
        #[arg(add = ArgValueCompleter::new(complete_remotes))]
        name: String,
    },

    Status {},

    /// Print the paths used by MID for its configuration and history files.
    Info {},

    List {
        #[arg(short, long, add = ArgValueCompleter::new(complete_tables))]
        table_name: Option<String>,
        #[arg(long, value_enum, default_value = "table")]
        output_format: QueryOutputFormat,
    },

    #[command(args_conflicts_with_subcommands = true, arg_required_else_help = true)]
    Query {
        #[arg()]
        query: Option<String>,
        #[arg(long, value_enum, default_value = "table")]
        output_format: QueryOutputFormat,
        #[command(subcommand)]
        command: Option<QueryCommands>,
        #[arg(long)]
        id: Option<u16>,
    },

    #[command(arg_required_else_help = true)]
    History {
        #[command(subcommand)]
        command: Option<HistoryCommands>,
    },

    Generator {
        #[arg(long)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum HistoryCommands {
    #[command(name = "last", visible_alias = "-", alias = "Last")]
    Last,
    List,
    Edit,
}

#[derive(Subcommand)]
pub enum QueryCommands {
    #[command(name = "last", visible_alias = "-", alias = "Last")]
    Last {
        #[arg(long, value_enum, default_value = "table")]
        output_format: QueryOutputFormat,
    },
}

#[derive(Subcommand)]
pub enum RemoteCommands {
    List,
    Add {
        #[arg()]
        connection_string: Option<String>,

        #[arg(short, long)]
        name: String,

        #[arg(short = 't', long)]
        database_type: Option<String>,

        #[arg(short = 's', long)]
        is_secure: bool,
    },
    Remove {
        #[arg(add = ArgValueCompleter::new(complete_remotes))]
        name: String,
    },
    Switch {
        #[arg(add = ArgValueCompleter::new(complete_remotes))]
        name: String,
    },
    Edit,
    Retrieve {
        #[arg()]
        name: String,
    },
    Password {
        #[arg()]
        name: String,
        #[arg()]
        password: String,
    },
    #[command(arg_required_else_help = true)]
    Local {
        #[command(subcommand)]
        command: Option<LocalCommands>,
    },
}

#[derive(Subcommand)]
pub enum LocalCommands {
    Init,
}
