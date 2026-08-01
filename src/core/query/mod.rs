mod app;
mod json;

#[derive(Debug, Clone)]
pub enum TableEvent {
    EditQuery(String),
    SelectTable(String),
}

use std::{
    env, fs,
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

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

#[derive(Debug, Clone, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum TableCommand {
    #[default]
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
    table_command: Option<TableCommand>,
) -> Result<Option<TableEvent>, Error> {
    return Ok(execute(query, options, table_command).await?);
}

async fn execute(
    query: String,
    options: QueryOutputFormat,
    table_command: Option<TableCommand>,
) -> Result<Option<TableEvent>, Error> {
    match options {
        QueryOutputFormat::Table => {
            let command = table_command.unwrap_or_default();
            let mut current_query = query;
            let items = query::execute_query_on_database(query::RunQueryOnDatabaseCommandOptions {
                query: current_query.clone(),
            })
            .await?;
            let mut app = App::new(items, command, current_query.clone());

            loop {
                let event =
                    ratatui::run(|terminal| app.run(terminal)).map_err(color_eyre::Report::from)?;

                match event {
                    Some(TableEvent::EditQuery(query)) => {
                        let Some(edited_query) = edit_query(&query)? else {
                            return Ok(None);
                        };
                        current_query = edited_query;
                        let items = query::execute_query_on_database(
                            query::RunQueryOnDatabaseCommandOptions {
                                query: current_query.clone(),
                            },
                        )
                        .await?;
                        app.update_query_results(items, current_query.clone());
                    }
                    event => return Ok(event),
                }
            }
        }
        QueryOutputFormat::Json => {
            let items =
                query::execute_query_on_database(query::RunQueryOnDatabaseCommandOptions { query })
                    .await?;

            render_output_as_json(items);
            Ok(None)
        }
    }
}

fn edit_query(query: &str) -> color_eyre::Result<Option<String>> {
    let (path, mut file) = create_query_temp_file()?;
    file.write_all(query.as_bytes())?;
    file.flush()?;
    drop(file);

    let result = (|| -> io::Result<Option<String>> {
        let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let mut editor_parts = editor.split_whitespace();
        let program = editor_parts.next().unwrap_or("vi");
        let status = Command::new(program)
            .args(editor_parts)
            .arg(&path)
            .status()?;
        if !status.success() {
            return Ok(None);
        }

        Ok(Some(fs::read_to_string(&path)?))
    })();

    let _ = fs::remove_file(&path);
    Ok(result?)
}

fn create_query_temp_file() -> io::Result<(PathBuf, fs::File)> {
    for attempt in 0..100 {
        let path = env::temp_dir().join(format!("mid-query-{}-{attempt}.sql", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a temporary query file",
    ))
}
