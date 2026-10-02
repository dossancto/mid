use std::collections::HashMap;

use crate::core::{
    config::types::DatabaseConfig,
    databases::adapters::database_type::{DatabaseHandler, DbValue, Error, QueryResult},
    secret::secret_manager::SecretManager,
};

use super::methods::{
    execute_query::{execute_postgres_dml, execute_postgres_query},
    export_sql::generate_postgres_export,
    list_table::list_tables_postgres,
    select_table::select_table_postgres,
    update_table::update_table_postgres,
};

pub struct PostgresHandler {
    config: DatabaseConfig,
}

impl PostgresHandler {
    pub fn new(mut config: DatabaseConfig) -> Result<Self, Error> {
        if config.is_secure {
            config.connection_string = SecretManager::new(config.name.clone())
                .connection_string(&config.connection_string)?;
        }
        Ok(Self { config })
    }
}

impl DatabaseHandler for PostgresHandler {
    async fn execute_select(&self, query: &str) -> Result<QueryResult, Error> {
        execute_postgres_query(&self.config, query.to_owned()).await
    }

    async fn execute_dml(&self, query: &str) -> Result<(), Error> {
        execute_postgres_dml(&self.config, query.to_owned()).await
    }

    fn export(&self, table_name: &str, items: Vec<HashMap<String, DbValue>>) -> String {
        generate_postgres_export(table_name, items)
    }

    fn list_tables(&self) -> String {
        list_tables_postgres()
    }

    fn select(&self, table_name: &str) -> String {
        select_table_postgres(table_name)
    }

    fn update(
        &self,
        table_name: &str,
        id_column: &str,
        id: &DbValue,
        values: &[(&str, &DbValue)],
    ) -> String {
        update_table_postgres(table_name, id_column, id, values)
    }

    fn table_name(&self, table_name: &str) -> String {
        format!("\"{}\"", table_name.replace('"', "\"\""))
    }

    fn list_databases_query(&self) -> String {
        "SELECT datname AS database_name, pg_size_pretty(pg_database_size(datname)) AS size FROM pg_database WHERE datistemplate = false;".to_string()
    }
}
