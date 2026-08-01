use thiserror::Error;

use crate::core::{
    config::manage,
    databases::{
        adapters::{
            DatabaseType, mysql::query::update_table_mysql, postgres::query::update_table_postgres,
        },
        application::query::DbValue,
    },
    globals,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read config file: {0}")]
    CurrentConfigError(#[from] manage::Error),

    #[error("Failed to update table: unsupported database type")]
    UnsupportedDatabase,
}

pub fn update_database_table(
    table_name: &str,
    id_column: &str,
    id: &DbValue,
    column: &str,
    value: &DbValue,
) -> Result<String, Error> {
    let file_path = globals::get_global_config_file_path();
    let config = manage::read_config(file_path)?;

    match config.get_database_type() {
        Some(DatabaseType::Postgres) => Ok(update_table_postgres(
            table_name, id_column, id, column, value,
        )),
        Some(DatabaseType::MySQL) => {
            Ok(update_table_mysql(table_name, id_column, id, column, value))
        }
        Some(DatabaseType::SQLite) | None => Err(Error::UnsupportedDatabase),
    }
}
