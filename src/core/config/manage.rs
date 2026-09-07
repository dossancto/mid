use std::{fs, io};
use thiserror::Error;

use crate::core::globals::{get_current_config_file_path, get_global_config_file_path};

use super::types::{DatabaseConfig, MidConfigFile};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse global config file: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Failed to serialize global config file: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("Failed to read global config file: {0}")]
    Io(#[from] io::Error),

    #[error("Database config already exists: {0}")]
    DatabaseAlreadyExists(String),

    #[error("Database config not found: {0}")]
    DatabaseNotFound(String),

    #[error("Database in use active cannot be removed: {0}")]
    DatabaseInUseCannotBeRemoved(String),
}

pub fn init_local_config() -> Result<MidConfigFile, Error> {
    let local_path = get_current_config_file_path()?;
    if let Some(local) = read_file(&local_path)? {
        return Ok(local);
    }

    let local = MidConfigFile::default();
    let contents = toml::to_string_pretty(&local)?;
    if let Some(parent) = std::path::Path::new(&local_path).parent() {
        fs::create_dir_all(parent)?;
    }
    // Do not overwrite a config created since the initial read.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&local_path)?;
    io::Write::write_all(&mut file, contents.as_bytes())?;
    Ok(local)
}

fn read_file(path: &str) -> Result<Option<MidConfigFile>, Error> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(toml::from_str(&contents)?)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn merge_configs(mut global: MidConfigFile, local: Option<MidConfigFile>) -> MidConfigFile {
    if let Some(local) = local {
        global
            .databases
            .retain(|database| !local.connection_exists(&database.name));
        global.databases.extend(local.databases);
        global.active_remote = local.active_remote.or(global.active_remote);
    }
    global
}

pub fn read_config() -> Result<MidConfigFile, Error> {
    let local_path = get_current_config_file_path()?;
    if let Some(local) = read_file(&local_path)? {
        return Ok(local);
    }
    Ok(read_file(&get_global_config_file_path())?.unwrap_or_default())
}

pub fn read_config_all() -> Result<MidConfigFile, Error> {
    let local_path = get_current_config_file_path()?;
    let local = read_file(&local_path)?;
    let global_path = get_global_config_file_path();
    if local_path == global_path {
        return Ok(local.unwrap_or_default());
    }
    let global = read_file(&global_path)?.unwrap_or_default();
    Ok(merge_configs(global, local))
}

pub fn save_config(content: MidConfigFile) -> Result<(), Error> {
    let local_path = get_current_config_file_path()?;
    save_to_paths(content, &local_path, get_global_config_file_path)
}

fn save_to_paths(
    content: MidConfigFile,
    local_path: &str,
    global_path: impl FnOnce() -> String,
) -> Result<(), Error> {
    let contents = toml::to_string_pretty(&content)?;
    if std::path::Path::new(local_path).try_exists()? {
        fs::write(local_path, contents)?;
        return Ok(());
    }
    fs::write(global_path(), contents)?;
    Ok(())
}

pub fn add_database(database: DatabaseConfig) -> Result<(), Error> {
    let mut config = read_config()?;

    if config.connection_exists(&database.name) {
        return Err(Error::DatabaseAlreadyExists(database.name));
    }

    config.databases.push(database);

    save_config(config)?;

    return Ok(());
}

pub fn remove_database(name: String) -> Result<(), Error> {
    let mut config = read_config()?;

    if !config.connection_exists(&name) {
        return Err(Error::DatabaseNotFound(name));
    }

    if config.active_remote.as_ref() == Some(&name) {
        return Err(Error::DatabaseInUseCannotBeRemoved(name));
    }

    config.databases.retain(|database| database.name != name);

    save_config(config)?;

    return Ok(());
}

pub fn retrieve_database(name: String) -> Result<DatabaseConfig, Error> {
    let config = read_config()?;

    if !config.connection_exists(&name) {
        return Err(Error::DatabaseNotFound(name));
    }

    let database = config.databases.iter().find(|db| db.name == name).unwrap();
    return Ok(database.clone());
}

pub fn read_databases() -> Result<Vec<DatabaseConfig>, Error> {
    let config = read_config_all()?;
    return Ok(config.databases);
}

pub fn change_active_database(name: String) -> Result<(), Error> {
    if !read_config_all()?.connection_exists(&name) {
        return Err(Error::DatabaseNotFound(name));
    }

    let mut config = read_config()?;
    config.set_active_database(name);

    save_config(config)?;

    return Ok(());
}
