use std::collections::{BTreeSet, HashMap};

use crate::core::databases::adapters::database_type::DbValue;

pub fn generate_postgres_export(table_name: &str, items: Vec<HashMap<String, DbValue>>) -> String {
    if items.is_empty() {
        return String::new();
    }

    fn identifier(value: &str) -> String {
        format!("\"{}\"", value.replace('"', "\"\""))
    }

    fn literal(value: &DbValue) -> String {
        match value {
            DbValue::Null => "NULL".to_string(),
            DbValue::Text(value) => format!("'{}'", value.replace('\'', "''")),
            DbValue::DateTime(value) => format!("'{}'", value.to_string().replace('\'', "''")),
            DbValue::TextArray(values) => format!(
                "ARRAY[{}]",
                values
                    .iter()
                    .map(|value| format!("'{}'", value.replace('\'', "''")))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            DbValue::Json(value) => format!("'{}'", value.replace('\'', "''")),
            DbValue::Numeric(value) => value.clone(),
            DbValue::Integer(value) => value.to_string(),
            DbValue::Float(value) if value.is_finite() => value.to_string(),
            DbValue::Float(_) => "NULL".to_string(),
            DbValue::Boolean(value) => value.to_string().to_uppercase(),
        }
    }

    let columns = items
        .iter()
        .flat_map(|row| row.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if columns.is_empty() {
        return String::new();
    }

    let column_list = columns
        .iter()
        .map(|column| identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let values = items
        .iter()
        .map(|row| {
            let values = columns
                .iter()
                .map(|column| {
                    row.get(column)
                        .map(literal)
                        .unwrap_or_else(|| "NULL".to_string())
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("({values})")
        })
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "INSERT INTO {} ({}) VALUES\n{};",
        identifier(table_name),
        column_list,
        values
    )
}
