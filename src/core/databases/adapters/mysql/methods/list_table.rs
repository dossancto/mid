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
