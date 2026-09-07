use serde::{Deserialize, Serialize};

use crate::core::databases::adapters::{
    database_type::{DatabaseType, Error},
    mysql::mysql_handler::MySqlHandler,
    postgres::postgres_handler::PostgresHandler,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct MidConfigFile {
    pub active_remote: Option<String>,
    pub databases: Vec<DatabaseConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub connection_string: String,
    #[serde(default)]
    pub is_secure: bool,
}

impl DatabaseConfig {
    pub fn get_database_type(&self) -> Result<DatabaseType, Error> {
        let database_type = self
            .connection_string
            .split(':')
            .next()
            .ok_or(Error::DatabaseTypeNotFound)?;

        match database_type {
            "postgres" | "postgresql" => {
                Ok(DatabaseType::Postgres(PostgresHandler::new(self.clone())?))
            }
            "mysql" => Ok(DatabaseType::MySQL(MySqlHandler::new(self.clone())?)),
            "sqlite" => Ok(DatabaseType::SQLite()),
            _ => Err(Error::DatabaseTypeNotFound),
        }
    }
}

impl Default for MidConfigFile {
    fn default() -> Self {
        Self {
            active_remote: None,
            databases: Vec::new(),
        }
    }
}

impl MidConfigFile {
    pub fn get_active_database(&self) -> Option<&DatabaseConfig> {
        let active_db_name = self.active_remote.as_ref()?;

        self.databases.iter().find(|db| db.name == *active_db_name)
    }

    pub fn connection_exists(&self, name: &str) -> bool {
        self.databases.iter().any(|db| db.name == name)
    }

    pub fn set_active_database(&mut self, name: String) {
        self.active_remote = Some(name);
    }

    pub fn get_database_type(&self) -> Result<DatabaseType, Error> {
        let active_db = self
            .get_active_database()
            .ok_or(Error::NoActiveRemoteConnection)?;

        let database_type = active_db
            .connection_string
            .split(':')
            .next()
            .ok_or(Error::DatabaseTypeNotFound)?;

        match database_type {
            "postgres" | "postgresql" => Ok(DatabaseType::Postgres(
                crate::core::databases::adapters::postgres::postgres_handler::PostgresHandler::new(
                    active_db.clone(),
                )?,
            )),
            "mysql" => Ok(DatabaseType::MySQL(
                crate::core::databases::adapters::mysql::mysql_handler::MySqlHandler::new(
                    active_db.clone(),
                )?,
            )),
            "sqlite" => Ok(DatabaseType::SQLite()),
            _ => Err(Error::DatabaseTypeNotFound),
        }
    }
}
