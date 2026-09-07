use crate::core::{
    self, editor::open_editor::open_editor_in_file, globals::get_global_history_file_path,
};

pub fn list() {
    let file_path = core::globals::get_global_history_file_path();
    let res = core::history::read_history(file_path);

    match res {
        Ok(history) => {
            for request in history.requests {
                println!("id: {}", request.id);
                println!("query: {}", request.query);
                println!("created_at: {}", request.created_at);
                println!("database: {}", request.database);
                println!("history_type: {:?}", request.history_type);
                println!("is_success: {}", request.is_success);
                println!("duration: {}", request.duration);
                println!("");
            }
        }
        Err(e) => eprintln!("No history found: {e}"),
    }
}

pub fn last() {
    let config = match core::config::manage::read_config_all() {
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

    let file_path = core::globals::get_global_history_file_path();
    let res = core::history::read_history(file_path);

    match res {
        Ok(history) => match history
            .requests
            .iter()
            .rev()
            .find(|request| request.database == active_database.name)
        {
            Some(request) => {
                println!("id: {}", request.id);
                println!("query: {}", request.query);
                println!("created_at: {}", request.created_at);
                println!("database: {}", request.database);
                println!("");
            }
            _ => println!("No history found for active database"),
        },
        Err(e) => eprintln!("No history found: {e}"),
    }
}

pub fn edit() {
    let file_path = get_global_history_file_path();
    if let Err(error) = open_editor_in_file(std::path::Path::new(&file_path)) {
        eprintln!("Failed to open history file: {error}");
    }
}
