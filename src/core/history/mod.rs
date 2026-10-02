use std::{fs, io};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct MidHistoryFile {
    pub requests: Vec<HistoryRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryRequest {
    pub id: u16,
    pub query: String,
    pub database: String,
    pub created_at: String,
    pub is_success: bool,
    pub duration: u64,
    pub history_type: HistoryRequestType,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum HistoryRequestType {
    DQL,
    DML,
}

impl Default for HistoryRequest {
    fn default() -> Self {
        Self {
            id: 0,

            query: String::new(),
            database: String::new(),
            created_at: String::new(),
            is_success: false,
            duration: 0,
            history_type: HistoryRequestType::DQL,
        }
    }
}

impl Default for MidHistoryFile {
    fn default() -> Self {
        Self {
            requests: Vec::new(),
        }
    }
}

impl MidHistoryFile {
    pub fn request_exists(&self, id: u16) -> bool {
        self.requests.iter().any(|request| request.id == id)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse global history file: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Failed to serialize global history file: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("Failed to read global history file: {0}")]
    Io(#[from] io::Error),

    #[error("History request not found: {0}")]
    RequestNotFound(u16),
}

pub fn read_history(file_path: String) -> Result<MidHistoryFile, Error> {
    let contents = match fs::read_to_string(file_path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(MidHistoryFile::default()),
        Err(e) => return Err(Error::Io(e)),
    };

    let history = toml::from_str::<MidHistoryFile>(&contents)?;

    return Ok(history);
}

pub fn get_history_id(file_path: String, id: &u16) -> Result<Option<HistoryRequest>, Error> {
    let history = read_history(file_path)?;
    let request = history.requests.iter().find(|r| r.id == *id);
    Ok(request.cloned())
}

pub fn save_history(file_path: String, content: MidHistoryFile) -> Result<(), Error> {
    let history_string = toml::to_string_pretty(&content)?;
    fs::write(file_path, history_string)?;

    return Ok(());
}

pub fn add_request(
    file_path: String,
    query: String,
    database: String,
    created_at: String,
    is_success: bool,
    duration: u64,
    history_type: HistoryRequestType,
) -> Result<(), Error> {
    let mut history = read_history(file_path.clone())?;
    let id = history
        .requests
        .iter()
        .map(|request| request.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);

    history.requests.push(HistoryRequest {
        id,
        query,
        database,
        created_at,
        is_success,
        duration,
        history_type,
    });
    save_history(file_path, history)
}

#[allow(dead_code)]
pub fn remove_request(file_path: String, id: u16) -> Result<(), Error> {
    let mut history = read_history(file_path.clone())?;

    if !history.request_exists(id) {
        return Err(Error::RequestNotFound(id));
    }

    history.requests.retain(|request| request.id != id);

    save_history(file_path, history)?;

    return Ok(());
}
