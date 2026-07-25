use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use sqlx::{
    AssertSqlSafe, Column, Row, TypeInfo, ValueRef,
    postgres::PgPoolOptions,
    types::{Uuid, chrono},
};
use thiserror::Error;

use crate::core::{config::types::DatabaseConfig, databases::application::query::DbValue};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to execute query: {0}")]
    SqlError(#[from] sqlx::Error),
}

/// Use this method to run an arbitrary query on the active database connection.
pub async fn execute_postgres_query(
    config: &DatabaseConfig,
    query: String,
) -> Result<Vec<HashMap<String, DbValue>>, Error> {
    let pool = PgPoolOptions::new()
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
                        "UUID" => row
                            .try_get::<Uuid, _>(column_name)
                            .map(|u| DbValue::Text(u.to_string()))
                            .unwrap_or(DbValue::Null),
                        "TIMESTAMP" | "TIMESTAMPTZ" => {
                            let decode_timestamp = || match catch_unwind(AssertUnwindSafe(|| {
                                row.try_get::<chrono::DateTime<chrono::Utc>, _>(column_name)
                            })) {
                                Ok(Ok(dt)) => DbValue::Text(dt.to_rfc3339()),
                                Ok(Err(_)) => DbValue::Null,
                                Err(_) => DbValue::Text("<timestamp out of range>".to_string()),
                            };

                            match value_ref.as_bytes() {
                                Ok(b"-infinity") => DbValue::Text("-infinity".to_string()),
                                Ok(b"infinity") => DbValue::Text("infinity".to_string()),
                                Ok(bytes) if bytes.len() == 8 => {
                                    let mut raw = [0_u8; 8];
                                    raw.copy_from_slice(bytes);

                                    match i64::from_be_bytes(raw) {
                                        i64::MIN => DbValue::Text("-infinity".to_string()),
                                        i64::MAX => DbValue::Text("infinity".to_string()),
                                        _ => decode_timestamp(),
                                    }
                                }
                                _ => decode_timestamp(),
                            }
                        }
                        "VARCHAR" | "TEXT" | "BPCHAR" | "NAME" => row
                            .try_get::<String, _>(column_name)
                            .map(DbValue::Text)
                            .unwrap_or(DbValue::Null),
                        "_TEXT" | "TEXT[]" => row
                            .try_get::<Vec<String>, _>(column_name)
                            .map(DbValue::TextArray)
                            .unwrap_or(DbValue::Null),
                        "NUMERIC" => row
                            .try_get::<String, _>(column_name)
                            .map(DbValue::Numeric)
                            .unwrap_or(DbValue::Null),
                        "INT2" | "INT4" | "INTEGER" => row
                            .try_get::<i32, _>(column_name)
                            .map(|n| DbValue::Integer(n as i64))
                            .unwrap_or(DbValue::Null),
                        "INT8" | "BIGINT" => row
                            .try_get::<i64, _>(column_name)
                            .map(DbValue::Integer)
                            .unwrap_or(DbValue::Null),
                        "BOOL" | "BOOLEAN" => row
                            .try_get::<bool, _>(column_name)
                            .map(DbValue::Boolean)
                            .unwrap_or(DbValue::Null),
                        "FLOAT4" | "REAL" => row
                            .try_get::<f32, _>(column_name)
                            .map(|n| DbValue::Float(n as f64))
                            .unwrap_or(DbValue::Null),
                        "FLOAT8" | "DOUBLE PRECISION" => row
                            .try_get::<f64, _>(column_name)
                            .map(DbValue::Float)
                            .unwrap_or(DbValue::Null),
                        // Safe fallback for complex types (Timestamps, UUIDs, JSON columns)
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

pub fn list_tables_postgres() -> String {
    return "
        SELECT table_name
        FROM information_schema.tables
        WHERE table_type = 'BASE TABLE'
          AND table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY table_schema, table_name;
        "
    .to_string();
}

pub fn select_table_postgres(table_name: &str) -> String {
    let table_name = table_name.replace('"', "\"\"");
    format!("SELECT * FROM \"{table_name}\"")
}

#[cfg(test)]
mod tests {
    use super::select_table_postgres;

    #[test]
    fn quotes_table_name_as_postgres_identifier() {
        assert_eq!(
            select_table_postgres("user\"data"),
            "SELECT * FROM \"user\"\"data\""
        );
    }
}
