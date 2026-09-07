use thiserror::Error;

use crate::{
    app::remote_add_screen::RemoteAddScreen,
    core::{
        config::{manage, types::DatabaseConfig},
        editor::open_editor::{open_editor_in_file, open_editor_recover_text},
        globals::get_global_config_file_path,
        secret::secret_manager::SecretManager,
    },
};

#[derive(Error, Debug)]
enum Error {
    #[error("Invalid connection URL or password encoding")]
    InvalidConnection,

    #[error("Connection URL must contain a password")]
    MissingPassword,

    #[error("database type not supported")]
    DatabaseTypeNotSupported,
}

pub fn edit() {
    let file_path = get_global_config_file_path();
    if let Err(error) = open_editor_in_file(std::path::Path::new(&file_path)) {
        eprintln!("Failed to open remote config: {error}");
    }
}

pub fn list() {
    let file_path = get_global_config_file_path();
    let res = manage::read_databases(file_path.to_owned());

    match res {
        Ok(databases) => {
            println!("Databases: ");
            for database in databases {
                println!(" {}", database.name);
            }
        }
        Err(e) => eprintln!("Failed to list remote configs: {e}"),
    }
}

pub fn remove(name: &str) {
    let file_path = get_global_config_file_path();
    let res = manage::remove_database(file_path.to_owned(), name.to_owned());

    match res {
        Ok(_) => println!("Remote config removed successfully. Database: {}", name),
        Err(e) => eprintln!("Failed to remove remote config: {e}"),
    }

    return;
}

pub fn retrieve(name: &str) {
    let file_path = get_global_config_file_path();
    let database = match manage::retrieve_database(file_path, name.to_owned()) {
        Ok(database) => database,
        Err(error) => {
            eprintln!("Failed to retrieve remote config: {error}");
            return;
        }
    };

    let connection_string = if database.is_secure {
        match SecretManager::new(database.name).connection_string(&database.connection_string) {
            Ok(connection_string) => connection_string,
            Err(_) => {
                eprintln!("Failed to retrieve connection string from the OS secret manager");
                return;
            }
        }
    } else {
        database.connection_string
    };

    println!("{connection_string}");
}

pub fn switch(name: &str) {
    let file_path = get_global_config_file_path();
    let res = manage::change_active_database(file_path.to_owned(), name.to_owned());

    match res {
        Ok(_) => println!("Switched active connection to {}", name),
        Err(e) => eprintln!("Failed to switch active connection: {e}"),
    }

    return;
}

pub fn password(name: &str, password: &str) {
    let file_path = get_global_config_file_path();

    let database = match manage::retrieve_database(file_path, name.to_owned()) {
        Ok(database) => database,
        Err(error) => {
            eprintln!("Failed to retrieve database: {error}");
            return;
        }
    };

    if !database.is_secure {
        eprintln!("Cannot set password: remote '{name}' is not secure");
        return;
    }

    let result = SecretManager::new(name.to_owned()).save_password(password.to_owned());

    match result {
        Ok(_) => println!("Password set for {}", name),
        Err(e) => eprintln!("Failed to set password: {e}"),
    }

    return;
}

pub fn add(
    name: &str,
    connection_string: Option<&str>,
    database_type: Option<&str>,
    is_secure: bool,
) {
    let mut connection_string = match connection_string {
        Some(connection_string) => connection_string.to_owned(),
        None if database_type.is_some() => {
            let template = match database_type {
                Some("mysql") => "mysql://username:password@localhost:3306/database",
                Some("postgres" | "postgresql") => {
                    "postgres://username:password@localhost:5432/database"
                }
                error => {
                    let error = error.unwrap_or_default().to_string();
                    eprintln!("Unsupported database type: {error}");
                    return;
                }
            };

            match open_editor_recover_text(template) {
                Ok(Some(connection_string)) if !connection_string.trim().is_empty() => {
                    connection_string.trim().to_owned()
                }
                Ok(_) => return,
                Err(error) => {
                    eprintln!("Failed to open connection string editor: {error}");
                    return;
                }
            }
        }
        None => {
            let mut screen = RemoteAddScreen::new();
            match ratatui::run(|terminal| screen.run(terminal)) {
                Ok(Some(connection_string)) => connection_string,
                Ok(None) => return,
                Err(error) => {
                    eprintln!("Failed to open remote connection form: {error}");
                    return;
                }
            }
        }
    };

    if is_secure {
        let sanitized = match sanitize_connection_string(&connection_string) {
            Ok(result) => result,
            Err(error) => {
                eprintln!("Error to extract password: {error}");
                return;
            }
        };
        let password = match extract_password(&connection_string) {
            Ok(password) => password,
            Err(error) => {
                eprintln!("{error}");
                return;
            }
        };
        if SecretManager::new(name.to_owned())
            .save_password(password)
            .is_err()
        {
            eprintln!("Failed to save remote password in the os manager");
            return;
        }
        connection_string = sanitized;
    }

    let file_path = get_global_config_file_path();
    let res = manage::add_database(
        file_path.to_owned(),
        DatabaseConfig {
            name: name.to_owned(),
            connection_string,
            is_secure,
        },
    );

    match res {
        Ok(_) => println!("Remote config added successfully. Database: {}", name),
        Err(e) => eprintln!("Failed to add remote config: {e}"),
    }
}

fn sanitize_connection_string(connection_string: &str) -> Result<String, Error> {
    let mut url = url::Url::parse(connection_string).map_err(|_| Error::InvalidConnection)?;
    match url.scheme() {
        "mysql" | "postgres" | "postgresql" => {}
        _ => return Err(Error::DatabaseTypeNotSupported),
    }
    // Query parameters may override URL credentials in database drivers.
    if url.query_pairs().any(|(key, _)| key == "password") {
        return Err(Error::InvalidConnection);
    }
    url.password().ok_or(Error::MissingPassword)?;
    url.set_password(Some("placeholder"))
        .map_err(|_| Error::InvalidConnection)?;
    Ok(format!(
        "{}{pass}{}",
        &url[..url::Position::BeforePassword],
        &url[url::Position::AfterPassword..],
        pass = "{pass}"
    ))
}

fn extract_password(connection_string: &str) -> Result<String, Error> {
    let url = url::Url::parse(connection_string).map_err(|_| Error::InvalidConnection)?;
    percent_encoding::percent_decode_str(url.password().ok_or(Error::MissingPassword)?)
        .decode_utf8()
        .map(|value| value.into_owned())
        .map_err(|_| Error::InvalidConnection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_decoded_password_and_preserves_connection_details() {
        for scheme in ["mysql", "postgres", "postgresql"] {
            let url = sanitize_connection_string(&format!(
                "{scheme}://user:pass%23%40%3A%25@[::1]:5432/db?sslmode=require"
            ))
            .unwrap();
            assert_eq!(
                url,
                format!("{scheme}://user:{{pass}}@[::1]:5432/db?sslmode=require")
            );
        }
    }

    #[test]
    fn rejects_missing_password_and_query_credentials() {
        assert!(sanitize_connection_string("mysql://user@localhost/db").is_err());
        assert!(
            sanitize_connection_string("mysql://user:secret@localhost/db?password=other").is_err()
        );
        assert!(sanitize_connection_string("sqlite://localhost/db").is_err());
    }
}
