use std::collections::HashMap;

use thiserror::Error;

use crate::core::{
    config::manage,
    databases::adapters::{
        DatabaseType, mysql::query::execute_mysql_query, postgres::query::execute_postgres_query,
    },
    globals::{self, get_global_history_file_path},
    history::{HistoryRequest, add_request},
};

#[derive(Debug, Clone)]
pub enum DbValue {
    Null,
    Text(String),
    TextArray(Vec<String>),
    Numeric(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

pub struct RunQueryOnDatabaseCommandOptions {
    pub query: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read config file: {0}")]
    CurrentConfigError(#[from] manage::Error),

    #[error(
        "Cant find an active remote connection. Please set an active connection before running a query."
    )]
    NoActiveRemoteConnection,

    #[error("Failed to execute query")]
    FailedToExecuteQuery(),
}

pub async fn execute_query_on_database(
    options: RunQueryOnDatabaseCommandOptions,
) -> Result<Vec<HashMap<String, DbValue>>, Error> {
    let file_path = globals::get_global_config_file_path();
    let config = manage::read_config(file_path)?;

    let active_database = config.get_active_database();

    if active_database.is_none() {
        return Err(Error::NoActiveRemoteConnection);
    }

    let active_database = active_database.unwrap();

    let response = match config.get_database_type() {
        Some(database_type) => match database_type {
            DatabaseType::Postgres => {
                let res = execute_postgres_query(active_database, options.query.clone()).await;

                if res.is_err() {
                    eprintln!("Failed to execute query on PostgreSQL: {res:?}");
                    return Err(Error::FailedToExecuteQuery());
                }

                res.unwrap()
            }
            DatabaseType::MySQL => {
                let res = execute_mysql_query(active_database, options.query.clone()).await;

                if res.is_err() {
                    eprintln!("Failed to execute query on MySQL: {res:?}");
                    return Err(Error::FailedToExecuteQuery());
                }

                res.unwrap()
            }
            DatabaseType::SQLite => {
                panic!("sqlite adapter not implemented yet");
            }
        },
        _ => {
            return Err(Error::NoActiveRemoteConnection);
        }
    };

    let file_path = get_global_history_file_path();
    let history_response = add_request(
        file_path,
        HistoryRequest {
            id: create_history_request_id(),
            query: options.query,
            database: active_database.name.clone(),
        },
    );

    if history_response.is_err() {
        eprintln!("Failed to save history to database",);
    }

    return Ok(response);
}

fn create_history_request_id() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time is before UNIX epoch")
        .as_nanos();

    return timestamp.to_string();
}
