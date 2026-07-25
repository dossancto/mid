use crate::core::{self, query::QueryOutputFormat};
use clap::Subcommand;
#[derive(Subcommand)]
pub enum QueryCommands {
    Last {
        #[arg(short, long)]
        output_format: Option<QueryOutputFormat>,
    },
    History {},
}

pub async fn handle_query_command(
    command: &Option<QueryCommands>,
    query: &Option<String>,
    output_format: &Option<QueryOutputFormat>,
) -> () {
    match command {
        Some(QueryCommands::Last { output_format }) => {
            let file_path_history = core::globals::get_global_history_file_path();
            let last_request = core::history::read_history(file_path_history);

            match last_request {
                Ok(history) => match history.requests.last() {
                    Some(last) => {
                        let res = core::query::handle_query_command(
                            last.query.clone(),
                            output_format
                                .as_ref()
                                .unwrap_or(&QueryOutputFormat::Table)
                                .clone(),
                            None,
                        )
                        .await;

                        match res {
                            Ok(_) => {}
                            Err(e) => eprintln!("Failed to execute query command: {e}"),
                        }
                    }
                    _ => println!("No history found"),
                },
                Err(e) => println!("No history found {}", e),
            }
        }
        _ => {
            let Some(query) = query else {
                eprintln!("Failed to execute query command: query is required");
                return;
            };

            let res = core::query::handle_query_command(
                query.to_string(),
                output_format
                    .as_ref()
                    .unwrap_or(&QueryOutputFormat::Table)
                    .clone(),
                None,
            )
            .await;

            match res {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to execute query command: {e}"),
            }
        }
    };
}
