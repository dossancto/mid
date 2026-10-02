use std::collections::HashMap;

use crate::core::{
    config::types::DatabaseConfig,
    databases::adapters::database_type::{DatabaseHandler, DbValue, Error, QueryResult},
    secret::secret_manager::SecretManager,
};

use super::methods::{
    execute_query::{execute_mysql_dml, execute_mysql_query},
    export_sql::generate_mysql_export,
    list_table::list_tables_mysql,
    select_table::select_table_mysql,
    update_table::update_table_mysql,
};

pub struct MySqlHandler {
    config: DatabaseConfig,
}

impl MySqlHandler {
    pub fn new(mut config: DatabaseConfig) -> Result<Self, Error> {
        if config.is_secure {
            config.connection_string = SecretManager::new(config.name.clone())
                .connection_string(&config.connection_string)?;
        }
        Ok(Self { config })
    }
}

impl DatabaseHandler for MySqlHandler {
    async fn execute_select(&self, query: &str) -> Result<QueryResult, Error> {
        execute_mysql_query(&self.config, query.to_owned()).await
    }

    async fn execute_dml(&self, query: &str) -> Result<(), Error> {
        execute_mysql_dml(&self.config, query.to_owned()).await
    }

    fn export(&self, table_name: &str, items: Vec<HashMap<String, DbValue>>) -> String {
        generate_mysql_export(table_name, items)
    }

    fn list_tables(&self) -> String {
        list_tables_mysql()
    }

    fn select(&self, table_name: &str) -> String {
        select_table_mysql(table_name)
    }

    fn update(
        &self,
        table_name: &str,
        id_column: &str,
        id: &DbValue,
        values: &[(&str, &DbValue)],
    ) -> String {
        update_table_mysql(table_name, id_column, id, values)
    }

    fn table_name(&self, table_name: &str) -> String {
        format!("`{}`", table_name.replace('`', "``"))
    }

    fn list_databases_query(&self) -> String {
        "SELECT schema_name AS database_name,
        FROM information_schema.schemata;"
            .to_string()
    }
}
