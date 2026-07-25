use clap::Subcommand;

use crate::core::{self};

#[derive(Subcommand)]
pub enum HistoryCommands {
    Last,
    List,
}

pub async fn handle_history_command(command: &Option<HistoryCommands>) -> () {
    match command {
        Some(HistoryCommands::List {}) => {
            let file_path = core::globals::get_global_history_file_path();
            let res = core::history::read_history(file_path);

            match res {
                Ok(history) => {
                    for request in history.requests {
                        println!("id:{} query: {}", request.id, request.query);
                    }
                }
                Err(e) => eprintln!("No history found: {e}"),
            }
        }
        Some(HistoryCommands::Last {}) => {
            let file_path = core::globals::get_global_history_file_path();
            let res = core::history::read_history(file_path);

            match res {
                Ok(history) => match history.requests.last() {
                    Some(last) => eprintln!("id: {} query: {}", last.id, last.query),
                    _ => println!("No history found"),
                },
                Err(e) => eprintln!("No history found: {e}"),
            }
        }
        _ => {}
    };
}
