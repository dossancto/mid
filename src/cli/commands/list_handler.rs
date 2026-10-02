use crate::core::config::manage;
use crate::core::databases::adapters::database_type::{DatabaseHandler, Error};
use crate::core::query::{QueryOutputFormat, TableEvent};
use crate::core::query::{TableCommand, handler::handle_query_command};

pub async fn list(
    output_format: &QueryOutputFormat,
    table_name: &Option<String>,
) -> Result<(), Error> {
    if let Some(table_name) = table_name {
        handle_selected_table(table_name, output_format).await?;
        return Ok(());
    }

    let config = manage::read_config_all()?;
    let database = config.get_database_type()?;

    let query = database.list_tables();

    match handle_query_command(query, output_format.clone(), Some(TableCommand::ShowTables)).await {
        Ok(Some(TableEvent::SelectTable(table_name))) => {
            handle_selected_table(&table_name, output_format).await?;
            return Ok(());
        }
        Ok(_) => {}
        Err(e) => eprintln!("[List] Failed to list tables: {e}"),
    }

    Ok(())
}

async fn handle_selected_table(
    table_name: &str,
    output_format: &QueryOutputFormat,
) -> Result<(), Error> {
    let config = manage::read_config_all()?;
    let database = config.get_database_type()?;
    let query = database.select(table_name);

    if let Err(e) =
        handle_query_command(query, output_format.clone(), Some(TableCommand::ShowValue)).await
    {
        eprintln!("[List] Failed to query selected table: {e}");
    }

    Ok(())
}
