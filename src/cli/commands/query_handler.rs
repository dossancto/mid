use crate::core::{self, history::HistoryRequestType, query::QueryOutputFormat};

pub async fn last(output_format: &QueryOutputFormat) {
    let file_path_history = core::globals::get_global_history_file_path();
    let last_request = core::history::read_history(file_path_history);
    let config = match core::config::manage::read_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to read config: {e}");
            return;
        }
    };
    let active_database = match config.get_active_database() {
        Some(database) => database,
        None => {
            eprintln!("No active remote connection");
            return;
        }
    };

    match last_request {
        Ok(history) => match history.requests.iter().rev().find(|request| {
            request.database == active_database.name
                && request.history_type == HistoryRequestType::DQL
        }) {
            Some(last) => {
                let res = core::query::handler::handle_query_command(
                    last.query.clone(),
                    output_format.clone(),
                    None,
                )
                .await;

                match res {
                    Ok(_) => {}
                    Err(e) => eprintln!("Failed to execute query command: {e}"),
                }
            }
            _ => println!("No history found for active database"),
        },
        Err(e) => println!("No history found {}", e),
    }
}

pub async fn by_id(output_format: &QueryOutputFormat, id: &u16) {
    let file_path = core::globals::get_global_history_file_path();
    let history_by_id = core::history::get_history_id(file_path, id);

    let request = history_by_id
        .unwrap()
        .expect("history request not found {id}");

    let res = core::query::handler::handle_query_command(
        request.query.to_string(),
        output_format.clone(),
        None,
    )
    .await;

    match res {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to execute query command by id {id}: {e}"),
    }
}

pub async fn query(query: &Option<String>, output_format: &QueryOutputFormat) {
    let Some(query) = query else {
        eprintln!("Failed to execute query command: query is required");
        return;
    };

    let res =
        core::query::handler::handle_query_command(query.to_string(), output_format.clone(), None)
            .await;

    match res {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to execute query command: {e}"),
    }
}
