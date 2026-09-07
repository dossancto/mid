use std::{collections::HashMap, fmt};

use sqlx::types::chrono;
use thiserror::Error;

use crate::core::databases::adapters::{
    mysql::mysql_handler::MySqlHandler, postgres::postgres_handler::PostgresHandler,
};

/// This module contains the database connection and related functionality.
pub enum DatabaseType {
    /// The Postgres database type.
    Postgres(PostgresHandler),

    /// The MySQL database type.
    MySQL(MySqlHandler),

    /// The SQLite database type.
    SQLite(),
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Postgres(_) => "PostgreSQL",
            Self::MySQL(_) => "MySQL",
            Self::SQLite() => "SQLite",
        };

        formatter.write_str(name)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to retrieve the secure connection string from the OS secret manager")]
    SecretUnavailable,
    #[error("Failed to execute query: {0}")]
    SqlError(#[from] sqlx::Error),

    #[error("Failed to read config file: {0}")]
    CurrentConfigError(#[from] crate::core::config::manage::Error),

    #[error(
        "Cant find an active remote connection. Please set an active connection before running a query"
    )]
    NoActiveRemoteConnection,

    #[error("Failed to execute query")]
    FailedToExecuteQuery(),

    #[error("Database type not found")]
    DatabaseTypeNotFound,
}

#[derive(Debug, Clone)]
pub enum DbValue {
    Null,
    Text(String),
    TextArray(Vec<String>),
    Json(String),
    Numeric(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(chrono::DateTime<chrono::Utc>),
}

/// Column names in SELECT order and rows with values at matching indices.
#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<DbValue>>,
}

impl QueryResult {
    /// Compatibility for consumers that address values by unique column name.
    pub fn into_maps(self) -> Vec<HashMap<String, DbValue>> {
        self.rows
            .into_iter()
            .map(|row| self.headers.iter().cloned().zip(row).collect())
            .collect()
    }

    pub fn new() -> Self {
        Self::default()
    }
}

pub trait DatabaseHandler {
    async fn execute_select(&self, query: &str) -> Result<QueryResult, Error>;
    async fn execute_dml(&self, query: &str) -> Result<(), Error>;
    fn export(&self, table_name: &str, items: Vec<HashMap<String, DbValue>>) -> String;
    fn list_tables(&self) -> String;
    fn select(&self, table_name: &str) -> String;
    fn update(
        &self,
        table_name: &str,
        id_column: &str,
        id: &DbValue,
        values: &[(&str, &DbValue)],
    ) -> String;
    fn table_name(&self, table_name: &str) -> String;
}
