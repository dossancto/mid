pub mod formats;
pub mod handler;

#[derive(Debug, Clone)]
pub enum TableEvent {
    EditQuery(String),
    UpdateValue(String),
    SelectTable(String),
    OpenSelectedRow(String),
}
use std::io;

use thiserror::Error;

use crate::core::databases::adapters::database_type;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum QueryOutputFormat {
    Table,
    Json,
    Sql,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum TableCommand {
    #[default]
    ShowValue,
    ShowTables,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to run the database query: {0}")]
    Database(#[from] database_type::Error),

    #[error("Fail to render query results")]
    RenderTable(#[from] color_eyre::Report),

    #[error("Failed to open editor; ensure the program in $EDITOR is available in PATH")]
    OpenEditor(),

    #[error(
        "$EDITOR is not configured; set it to an editor available in PATH (for example: export EDITOR=nvim)"
    )]
    EditorNotConfigured(),

    #[error("Fail to create temporary query file")]
    CreateTempFile(#[from] io::Error),
}
