use keyring::Entry;
use keyring::Error;

pub struct SecretManager {
    name: String,
}
impl SecretManager {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn save_password(&mut self, password: String) -> Result<(), Error> {
        let service_name = "mid";
        let entry = Entry::new(service_name, &self.name)?;

        entry.set_password(&password)?;

        Ok(())
    }

    pub fn get_password(&self) -> Result<String, Error> {
        let service_name = "mid";
        let entry = Entry::new(service_name, &self.name)?;

        let password = entry.get_password()?;

        Ok(password)
    }
}
