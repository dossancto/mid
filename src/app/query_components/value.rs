use crate::core::databases::adapters::database_type::DbValue;

pub(crate) fn format_db_value(value: &DbValue) -> String {
    match value {
        DbValue::Null => "NULL".to_string(),
        DbValue::Text(value) => value.clone(),
        DbValue::DateTime(value) => value.to_string(),
        DbValue::TextArray(values) => format!("{{{}}}", values.join(",")),
        DbValue::Json(value) => value.clone(),
        DbValue::Numeric(value) => value.clone(),
        DbValue::Integer(value) => value.to_string(),
        DbValue::Float(value) if value.is_finite() => value.to_string(),
        DbValue::Float(_) => "NULL".to_string(),
        DbValue::Boolean(value) => value.to_string(),
    }
}

pub(crate) fn format_db_value_preview(value: &DbValue, max_characters: usize) -> String {
    if max_characters == 0 {
        return String::new();
    }

    match value {
        DbValue::Text(value) | DbValue::Json(value) | DbValue::Numeric(value) => {
            truncate_with_ellipsis(value, max_characters, false)
        }
        _ => truncate_with_ellipsis(&format_db_value(value), max_characters, false),
    }
}

fn truncate_with_ellipsis(value: &str, max_characters: usize, force_ellipsis: bool) -> String {
    let mut characters = value.chars();
    let mut output = characters.by_ref().take(max_characters).collect::<String>();
    let truncated = force_ellipsis || characters.next().is_some();

    if truncated {
        output.pop();
        output.push('…');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{DbValue, format_db_value_preview};

    #[test]
    fn bounds_text_preview() {
        let preview = format_db_value_preview(&DbValue::Text("abcdefghij".into()), 5);
        assert_eq!(preview, "abcd…");
    }

    #[test]
    fn bounds_json_preview() {
        let value = DbValue::Json(format!(r#"{{"large":"{}"}}"#, "x".repeat(10_000)));
        let preview = format_db_value_preview(&value, 50);

        assert!(preview.chars().count() <= 50);
        assert!(preview.ends_with('…'));
    }
}
