mod app;
mod json;
mod table;

pub use table::TableEvent;

use sqlx::Result;
use thiserror::Error;

use crate::core::{
    databases::application::query,
    query::{app::App, json::render_output_as_json},
};

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum QueryOutputFormat {
    Table,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum TableCommand {
    ShowValue,
    ShowTables,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Fail to run the query")]
    ExecuteQuery(#[from] query::Error),

    #[error("Fail to render query results")]
    RenderTable(#[from] color_eyre::Report),
}

pub async fn handle_query_command(
    query: String,
    options: QueryOutputFormat,
    table_commands: Option<Vec<TableCommand>>,
) -> Result<Option<TableEvent>, Error> {
    return Ok(execute(query, options, table_commands).await?);
}

async fn execute(
    query: String,
    options: QueryOutputFormat,
    table_command: Option<Vec<TableCommand>>,
) -> Result<Option<TableEvent>, Error> {
    let items = query::execute_query_on_database(query::RunQueryOnDatabaseCommandOptions {
        query: query.clone(),
    })
    .await?;

    match options {
        QueryOutputFormat::Table => {
            let mut app = App::new(
                items,
                table_command.unwrap_or(vec![TableCommand::ShowValue]),
                query,
            );

            ratatui::run(|terminal| app.run(terminal)).map_err(color_eyre::Report::from)?;
            Ok(None)
        }
        QueryOutputFormat::Json => {
            render_output_as_json(items);
            Ok(None)
        }
    }
}
