use std::{fs, io};
use thiserror::Error;

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

pub fn read_config(file_path: String) -> Result<MidConfigFile, Error> {
    let contents = match fs::read_to_string(file_path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(MidConfigFile::default()),
        Err(e) => return Err(Error::Io(e)),
    };

    let config = toml::from_str::<MidConfigFile>(&contents)?;

    return Ok(config);
}

pub fn save_config(file_path: String, content: MidConfigFile) -> Result<(), Error> {
    let config_string = toml::to_string_pretty(&content)?;
    fs::write(file_path, config_string)?;

    return Ok(());
}

pub fn add_database(file_path: String, database: DatabaseConfig) -> Result<(), Error> {
    let mut config = read_config(file_path.clone())?;

    if config.connection_exists(&database.name) {
        return Err(Error::DatabaseAlreadyExists(database.name));
    }

    config.databases.push(database);

    save_config(file_path, config)?;

    return Ok(());
}

pub fn remove_database(file_path: String, name: String) -> Result<(), Error> {
    let mut config = read_config(file_path.clone())?;

    if !config.connection_exists(&name) {
        return Err(Error::DatabaseNotFound(name));
    }

    if config.active_remote.as_ref() == Some(&name) {
        return Err(Error::DatabaseInUseCannotBeRemoved(name));
    }

    config.databases.retain(|database| database.name != name);

    save_config(file_path, config)?;

    return Ok(());
}

pub fn retrieve_database(file_path: String, name: String) -> Result<DatabaseConfig, Error> {
    let config = read_config(file_path)?;

    if !config.connection_exists(&name) {
        return Err(Error::DatabaseNotFound(name));
    }

    let database = config.databases.iter().find(|db| db.name == name).unwrap();
    return Ok(database.clone());
}

pub fn read_databases(file_path: String) -> Result<Vec<DatabaseConfig>, Error> {
    let config = read_config(file_path)?;
    return Ok(config.databases);
}

pub fn change_active_database(file_path: String, name: String) -> Result<(), Error> {
    let mut config = read_config(file_path.clone())?;

    if !config.connection_exists(&name) {
        return Err(Error::DatabaseNotFound(name));
    }

    config.set_active_database(name);

    save_config(file_path, config)?;

    return Ok(());
}
