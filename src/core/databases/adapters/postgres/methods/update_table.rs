use crate::core::databases::adapters::database_type::DbValue;

pub fn update_table_postgres(
    table_name: &str,
    id_column: &str,
    id: &DbValue,
    values: &[(&str, &DbValue)],
) -> String {
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

    let assignments = values
        .iter()
        .map(|(column, value)| format!("{} = {}", identifier(column), literal(value)))
        .collect::<Vec<_>>()
        .join(",\n    ");

    format!(
        "UPDATE {}\nSET {}\nWHERE {} = {};",
        identifier(table_name),
        assignments,
        identifier(id_column),
        literal(id),
    )
}
