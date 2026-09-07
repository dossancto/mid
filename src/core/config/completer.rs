use std::ffi::OsStr;

use clap_complete::CompletionCandidate;

use crate::core::{
    config::manage,
    databases::adapters::database_type::{DatabaseHandler, DbValue},
};

pub fn complete_remotes(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();
    let Ok(config) = manage::read_config_all() else {
        return Vec::new();
    };
    let active_remote = config.active_remote;

    config
        .databases
        .into_iter()
        .filter(|database| database.name.starts_with(prefix.as_ref()))
        .map(|database| {
            let help = database.get_database_type().ok().map(|db_type| {
                if active_remote.as_deref() == Some(database.name.as_str()) {
                    format!("{db_type} (Active)").into()
                } else {
                    db_type.to_string().into()
                }
            });

            CompletionCandidate::new(database.name).help(help)
        })
        .collect()
}

pub fn complete_tables(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();
    let Ok(config) = manage::read_config_all() else {
        return Vec::new();
    };
    let Ok(database) = config.get_database_type() else {
        return Vec::new();
    };

    let query = database.list_tables();
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        return Vec::new();
    };
    let Ok(tables) =
        tokio::task::block_in_place(|| runtime.block_on(database.execute_select(&query)))
    else {
        return Vec::new();
    };

    tables
        .into_maps()
        .into_iter()
        .filter_map(|mut table| match table.remove("table_name") {
            Some(DbValue::Text(table_name)) if table_name.starts_with(prefix.as_ref()) => {
                Some(CompletionCandidate::new(table_name))
            }
            _ => None,
        })
        .collect()
}
