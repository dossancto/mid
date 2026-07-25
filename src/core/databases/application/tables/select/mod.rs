use thiserror::Error;

use crate::core::{
    config::manage,
    databases::adapters::{
        DatabaseType, mysql::query::select_table_mysql, postgres::query::select_table_postgres,
    },
    globals,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read config file: {0}")]
    CurrentConfigError(#[from] manage::Error),

    #[error("Failed to select table: unsupported database type")]
    UnsupportedDatabase,
}

pub fn select_database_table(table_name: &str) -> Result<String, Error> {
    let file_path = globals::get_global_config_file_path();
    let config = manage::read_config(file_path)?;

    match config.get_database_type() {
        Some(DatabaseType::Postgres) => Ok(select_table_postgres(table_name)),
        Some(DatabaseType::MySQL) => Ok(select_table_mysql(table_name)),
        Some(DatabaseType::SQLite) | None => Err(Error::UnsupportedDatabase),
    }
}
