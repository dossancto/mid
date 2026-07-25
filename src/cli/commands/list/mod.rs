use clap::Subcommand;
use crossterm::event::KeyCode;

use crate::core::{
    databases::application::tables,
    query::{QueryOutputFormat, TableCommand, handle_query_command},
};

#[derive(Subcommand)]
pub enum ListCommands {
    /// List for tables in the database
    Tables {
        #[arg()]
        table_name: String,
    },
}

pub async fn handle_list_command(
    //command: &Option<RemoteCommands>,
    output_format: QueryOutputFormat,
) -> () {
    let res = tables::list::list_database_tables();

    let query = res.unwrap_or_default();
    let res = handle_query_command(
        query,
        output_format.clone(),
        Some(vec![TableCommand::Moviment, TableCommand::DatabaseTable]),
    )
    .await;

    match res {
        Ok(Some(event)) if event.key_code == KeyCode::Enter => {
            if let Some(table_name) = event.value {
                let query = match tables::select::select_database_table(&table_name) {
                    Ok(query) => query,
                    Err(e) => {
                        eprintln!("[List] Failed to build query for selected table: {e}");
                        return;
                    }
                };
                let result =
                    handle_query_command(query, output_format, Some(vec![TableCommand::Moviment]))
                        .await;

                if let Err(e) = result {
                    eprintln!("[List] Failed to query selected table: {e}");
                }
            }
        }
        Ok(_) => {}
        Err(e) => eprintln!("[List] Failed to execute query command:  {e}"),
    }
}
