use crate::core::config::manage;

pub fn status() {
    let current_config = manage::read_config_all();

    match current_config {
        Ok(config) => {
            let active_remote = config.get_active_database();

            match active_remote {
                Some(remote) => println!("Active remote: {}", remote.name),
                None => println!("No active remote found in the config file"),
            }
        }
        Err(e) => eprintln!("Failed to get current config: {e}"),
    }
}
