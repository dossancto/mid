use std::env;

/// The config file name.
pub const CONFIG_FILE_NAME: &str = ".midconfig.toml";
pub const HISTORY_FILE_NAME: &str = ".midhistory.toml";

pub fn get_global_config_file_path() -> String {
    return get_global_file_path(CONFIG_FILE_NAME);
}

pub fn get_global_history_file_path() -> String {
    return get_cache_file_path(HISTORY_FILE_NAME);
}

fn get_global_file_path(file_name: &str) -> String {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let file_path = home_dir.join(file_name);

    return file_path.to_string_lossy().to_string();
}

fn get_cache_file_path(file_name: &str) -> String {
    let home_dir = env::temp_dir();
    let file_path = home_dir.join(file_name);

    return file_path.to_string_lossy().to_string();
}
