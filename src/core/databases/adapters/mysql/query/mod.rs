use std::collections::HashMap;

use sqlx::{AssertSqlSafe, Column, Row, TypeInfo, ValueRef, mysql::MySqlPoolOptions};
use thiserror::Error;

use crate::core::{config::types::DatabaseConfig, databases::application::query::DbValue};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to execute query: {0}")]
    SqlError(#[from] sqlx::Error),
}

/// Use this method to run an arbitrary query on the active database connection.
pub async fn execute_mysql_query(
    config: &DatabaseConfig,
    query: String,
) -> Result<Vec<HashMap<String, DbValue>>, Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&config.connection_string)
        .await?;

    // We cant assert the query is safe, but this will only affect the user database, so theres
    // no point to try to validate the query, since the user is the one writing it, and if they
    // write a malicious query, its their own fault, so we will just execute it as is.
    let safe_query = AssertSqlSafe(query);

    let rows = sqlx::query(safe_query).fetch_all(&pool).await?;

    pool.close().await;

    let mut parsed_rows = Vec::new();

    for row in &rows {
        let mut row_map = HashMap::new();

        for column in row.columns() {
            let column_name = column.name();

            let db_value = match row.try_get_raw(column_name) {
                Ok(value_ref) if !value_ref.is_null() => {
                    let type_name = column.type_info().name();
                    match type_name {
                        "VARCHAR" | "CHAR" | "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT"
                        | "ENUM" | "SET" | "JSON" | "DATE" | "TIME" | "DATETIME" | "TIMESTAMP"
                        | "YEAR" => row
                            .try_get::<String, _>(column_name)
                            .map(DbValue::Text)
                            .unwrap_or(DbValue::Null),
                        "DECIMAL" | "NEWDECIMAL" => row
                            .try_get::<String, _>(column_name)
                            .map(DbValue::Numeric)
                            .unwrap_or(DbValue::Null),
                        "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "INTEGER" => row
                            .try_get::<i32, _>(column_name)
                            .map(|n| DbValue::Integer(n as i64))
                            .unwrap_or(DbValue::Null),
                        "BIGINT" => row
                            .try_get::<i64, _>(column_name)
                            .map(DbValue::Integer)
                            .unwrap_or(DbValue::Null),
                        "BOOLEAN" | "BOOL" => row
                            .try_get::<bool, _>(column_name)
                            .map(DbValue::Boolean)
                            .unwrap_or(DbValue::Null),
                        "FLOAT" => row
                            .try_get::<f32, _>(column_name)
                            .map(|n| DbValue::Float(n as f64))
                            .unwrap_or(DbValue::Null),
                        "DOUBLE" => row
                            .try_get::<f64, _>(column_name)
                            .map(DbValue::Float)
                            .unwrap_or(DbValue::Null),
                        "BIT" => row
                            .try_get::<bool, _>(column_name)
                            .map(DbValue::Boolean)
                            .unwrap_or_else(|_| {
                                row.try_get::<i64, _>(column_name)
                                    .map(DbValue::Integer)
                                    .unwrap_or(DbValue::Null)
                            }),
                        "BINARY" | "VARBINARY" | "BLOB" | "TINYBLOB" | "MEDIUMBLOB"
                        | "LONGBLOB" => DbValue::Text("<binary>".to_string()),
                        _ => row
                            .try_get::<String, _>(column_name)
                            .map(DbValue::Text)
                            .unwrap_or_else(|_| {
                                DbValue::Text(format!("<unsupported: {}>", type_name))
                            }),
                    }
                }
                _ => DbValue::Null,
            };

            row_map.insert(column_name.to_string(), db_value);
        }

        parsed_rows.push(row_map);
    }

    return Ok(parsed_rows);
}

pub fn list_tables_mysql() -> String {
    return "
        SELECT CAST(table_name AS CHAR CHARACTER SET utf8mb4) AS table_name
        FROM information_schema.tables
        WHERE table_type = 'BASE TABLE'
          AND table_schema = DATABASE()
        ORDER BY table_name;
        "
    .to_string();
}

pub fn select_table_mysql(table_name: &str) -> String {
    let table_name = table_name.replace('`', "``");
    format!("SELECT * FROM `{table_name}`")
}

#[cfg(test)]
mod tests {
    use super::select_table_mysql;

    #[test]
    fn quotes_table_name_as_mysql_identifier() {
        assert_eq!(
            select_table_mysql("user`data"),
            "SELECT * FROM `user``data`"
        );
    }
}
