pub fn select_table_postgres(table_name: &str) -> String {
    let table_name = table_name.replace('"', "\"\"");
    format!("SELECT * FROM \"{table_name}\" LIMIT 1000")
}
