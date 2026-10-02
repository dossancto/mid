use sqlx::{
    AssertSqlSafe, Column, Row, TypeInfo, ValueRef,
    postgres::{PgPoolOptions, PgValueFormat},
    types::{Uuid, chrono},
};

use crate::core::config::types::DatabaseConfig;
use crate::core::databases::adapters::database_type::{DbValue, Error, QueryResult};

/// Use this method to run an arbitrary query on the active database connection.
pub async fn execute_postgres_query(
    config: &DatabaseConfig,
    query: String,
) -> Result<QueryResult, Error> {
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

    let headers = rows
        .first()
        .map(|row| {
            row.columns()
                .iter()
                .map(|column| column.name().to_owned())
                .collect()
        })
        .unwrap_or_default();
    let mut parsed_rows = Vec::new();

    for row in rows {
        let mut values = Vec::new();

        for (index_column, column) in row.columns().iter().enumerate() {
            let db_value = match row.try_get_raw(index_column) {
                Ok(value_ref) if !value_ref.is_null() => {
                    let type_name = column.type_info().name();
                    match type_name {
                        "UUID" => row
                            .try_get::<Uuid, _>(index_column)
                            .map(|u| DbValue::Text(u.to_string()))
                            .unwrap_or(DbValue::Null),
                        "TIMESTAMP" | "TIMESTAMPTZ" => {
                            match row.try_get::<String, _>(index_column) {
                                Ok(value) => match value.as_str() {
                                    "-infinity" | "infinity" => DbValue::Text(value),
                                    _ => row
                                        .try_get::<chrono::DateTime<chrono::Utc>, _>(index_column)
                                        .map(|t| DbValue::Text(t.to_string()))
                                        .unwrap_or(DbValue::Null),
                                },
                                Err(_) => postgres_infinity(value_ref)
                                    .or_else(|| {
                                        row.try_get::<chrono::DateTime<chrono::Utc>, _>(
                                            index_column,
                                        )
                                        .ok()
                                        .map(DbValue::DateTime)
                                    })
                                    .unwrap_or(DbValue::Null),
                            }
                        }
                        "VARCHAR" | "TEXT" | "BPCHAR" | "NAME" => row
                            .try_get::<String, _>(index_column)
                            .map(DbValue::Text)
                            .unwrap_or(DbValue::Null),
                        "_TEXT" | "TEXT[]" => row
                            .try_get::<Vec<String>, _>(index_column)
                            .map(DbValue::TextArray)
                            .unwrap_or(DbValue::Null),
                        "NUMERIC" => row
                            .try_get::<String, _>(index_column)
                            .map(DbValue::Numeric)
                            .unwrap_or(DbValue::Null),
                        "INT2" | "INT4" | "INTEGER" => row
                            .try_get::<i32, _>(index_column)
                            .map(|n| DbValue::Integer(n as i64))
                            .unwrap_or(DbValue::Null),
                        "INT8" | "BIGINT" => row
                            .try_get::<i64, _>(index_column)
                            .map(DbValue::Integer)
                            .unwrap_or(DbValue::Null),
                        "BOOL" | "BOOLEAN" => row
                            .try_get::<bool, _>(index_column)
                            .map(DbValue::Boolean)
                            .unwrap_or(DbValue::Null),
                        "JSONB" => postgres_jsonb(value_ref)
                            .map(DbValue::Json)
                            .unwrap_or(DbValue::Null),
                        "FLOAT4" | "REAL" => row
                            .try_get::<f32, _>(index_column)
                            .map(|n| DbValue::Float(n as f64))
                            .unwrap_or(DbValue::Null),
                        "FLOAT8" | "DOUBLE PRECISION" => row
                            .try_get::<f64, _>(index_column)
                            .map(DbValue::Float)
                            .unwrap_or(DbValue::Null),
                        // Safe fallback for complex types (Timestamps, UUIDs, JSON columns)
                        _ => row
                            .try_get::<String, _>(index_column)
                            .map(DbValue::Text)
                            .unwrap_or_else(|_| {
                                DbValue::Text(format!("<unsupported: {}>", type_name))
                            }),
                    }
                }
                _ => DbValue::Null,
            };

            values.push(db_value);
        }

        parsed_rows.push(values);
    }

    return Ok(QueryResult {
        headers,
        rows: parsed_rows,
    });
}

fn postgres_infinity(value: sqlx::postgres::PgValueRef<'_>) -> Option<DbValue> {
    match value.format() {
        PgValueFormat::Text => match value.as_str().ok()? {
            "-infinity" => Some(DbValue::Text("-infinity".to_owned())),
            "infinity" => Some(DbValue::Text("infinity".to_owned())),
            _ => None,
        },
        PgValueFormat::Binary => {
            let bytes = value.as_bytes().ok()?;
            if bytes == i64::MIN.to_be_bytes() || bytes == i32::MIN.to_be_bytes() {
                Some(DbValue::Text("-infinity".to_owned()))
            } else if bytes == i64::MAX.to_be_bytes() || bytes == i32::MAX.to_be_bytes() {
                Some(DbValue::Text("infinity".to_owned()))
            } else {
                None
            }
        }
    }
}

fn postgres_jsonb(value: sqlx::postgres::PgValueRef<'_>) -> Option<String> {
    match value.format() {
        PgValueFormat::Text => value.as_str().ok().map(str::to_owned),
        // PostgreSQL prefixes the JSONB binary representation with a version byte.
        PgValueFormat::Binary => {
            let bytes = value.as_bytes().ok()?;
            String::from_utf8(bytes.get(1..)?.to_vec()).ok()
        }
    }
}

/// Execute one or more data-modification statements without preparing them.
pub async fn execute_postgres_dml(config: &DatabaseConfig, query: String) -> Result<(), Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.connection_string)
        .await?;

    let result = sqlx::raw_sql(AssertSqlSafe(query)).execute(&pool).await;
    pool.close().await;
    result?;

    Ok(())
}
