use std::collections::HashMap;

use crate::core::databases::adapters::database_type::DbValue;

pub fn render_output_as_json(items: Vec<HashMap<String, DbValue>>) {
    let json_elements = format_to_json_elements(items);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::Value::Array(json_elements)).unwrap()
    );
}

fn format_to_json_elements(items: Vec<HashMap<String, DbValue>>) -> Vec<serde_json::Value> {
    let json_elements: Vec<serde_json::Value> = items
        .iter()
        .map(|row| {
            let mut map = serde_json::Map::new();
            for (k, v) in row {
                let json_val = match v {
                    DbValue::Null => serde_json::Value::Null,
                    DbValue::Text(s) => serde_json::Value::String(s.clone()),
                    DbValue::DateTime(value) => serde_json::Value::String(value.to_string()),
                    DbValue::TextArray(values) => serde_json::Value::Array(
                        values
                            .iter()
                            .map(|value| serde_json::Value::String(value.clone()))
                            .collect(),
                    ),
                    DbValue::Json(value) => serde_json::from_str(value)
                        .unwrap_or_else(|_| serde_json::Value::String(value.clone())),
                    DbValue::Numeric(value) => serde_json::Value::String(value.clone()),
                    DbValue::Integer(n) => serde_json::Value::Number((*n).into()),
                    DbValue::Float(f) => serde_json::Number::from_f64(*f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    DbValue::Boolean(b) => serde_json::Value::Bool(*b),
                };
                map.insert(k.clone(), json_val);
            }
            serde_json::Value::Object(map)
        })
        .collect();

    json_elements
}
