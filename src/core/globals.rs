use std::{env, fs, path::PathBuf};

use crate::core::config::manage::Error;

/// The config file name.
pub const CONFIG_FILE_NAME: &str = "mid/.midconfig.toml";
pub const HISTORY_FILE_NAME: &str = "mid/.midhistory.toml";

pub fn get_global_config_file_path() -> String {
    return get_global_file_path(CONFIG_FILE_NAME);
}

pub fn get_global_history_file_path() -> String {
    return get_cache_file_path(HISTORY_FILE_NAME);
}

pub fn get_current_config_file_path() -> Result<String, Error> {
    return get_current_file_path(CONFIG_FILE_NAME);
}

fn get_current_file_path(file_name: &str) -> Result<String, Error> {
    let current_dir = std::env::current_dir()?;
    Ok(current_dir.join(file_name).to_string_lossy().into_owned())
}

fn get_global_file_path(file_name: &str) -> String {
    let home_dir = dirs::config_dir().expect("Could not find home directory");
    return prepare_file_path(home_dir.join(file_name));
}

fn get_cache_file_path(file_name: &str) -> String {
    let home_dir = env::temp_dir();
    return prepare_file_path(home_dir.join(file_name));
}

fn prepare_file_path(file_path: PathBuf) -> String {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).expect("Could not create application directory");
    }

    return file_path.to_string_lossy().to_string();
}
