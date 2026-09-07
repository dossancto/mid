use crate::core::config::manage;

pub fn init() {
    manage::init_local_config().unwrap();
}
