use std::collections::HashMap;

use crate::core::databases::adapters::database_type::{
    DatabaseHandler, DatabaseType, DbValue, Error, QueryResult,
};

impl DatabaseHandler for DatabaseType {
    async fn execute_select(&self, query: &str) -> Result<QueryResult, Error> {
        match self {
            DatabaseType::Postgres(handler) => handler.execute_select(query).await,
            DatabaseType::MySQL(handler) => handler.execute_select(query).await,
            DatabaseType::SQLite() => todo!(),
        }
    }

    async fn execute_dml(&self, query: &str) -> Result<(), Error> {
        match self {
            DatabaseType::Postgres(handler) => handler.execute_dml(query).await,
            DatabaseType::MySQL(handler) => handler.execute_dml(query).await,
            DatabaseType::SQLite() => todo!(),
        }
    }

    fn export(&self, table_name: &str, items: Vec<HashMap<String, DbValue>>) -> String {
        match self {
            DatabaseType::Postgres(handler) => handler.export(table_name, items),
            DatabaseType::MySQL(handler) => handler.export(table_name, items),
            DatabaseType::SQLite() => todo!(),
        }
    }

    fn list_tables(&self) -> String {
        match self {
            DatabaseType::Postgres(handler) => handler.list_tables(),
            DatabaseType::MySQL(handler) => handler.list_tables(),
            DatabaseType::SQLite() => todo!(),
        }
    }

    fn select(&self, table_name: &str) -> String {
        match self {
            DatabaseType::Postgres(handler) => handler.select(table_name),
            DatabaseType::MySQL(handler) => handler.select(table_name),
            DatabaseType::SQLite() => todo!(),
        }
    }

    fn update(
        &self,
        table_name: &str,
        id_column: &str,
        id: &DbValue,
        values: &[(&str, &DbValue)],
    ) -> String {
        match self {
            DatabaseType::Postgres(handler) => handler.update(table_name, id_column, id, values),
            DatabaseType::MySQL(handler) => handler.update(table_name, id_column, id, values),
            DatabaseType::SQLite() => todo!(),
        }
    }

    fn table_name(&self, table_name: &str) -> String {
        match self {
            DatabaseType::Postgres(handler) => handler.table_name(table_name),
            DatabaseType::MySQL(handler) => handler.table_name(table_name),
            DatabaseType::SQLite() => todo!(),
        }
    }

    fn list_databases_query(&self) -> String {
        match self {
            DatabaseType::Postgres(handler) => handler.list_databases_query(),
            DatabaseType::MySQL(handler) => handler.list_databases_query(),
            DatabaseType::SQLite() => todo!(),
        }
    }
}
