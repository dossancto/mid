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

    pub fn connection_string(
        &self,
        template: &str,
    ) -> Result<String, crate::core::databases::adapters::database_type::Error> {
        use crate::core::databases::adapters::database_type::Error as DatabaseError;
        let password = self
            .get_password()
            .map_err(|_| DatabaseError::SecretUnavailable)?;
        restore_password(template, &password).map_err(|_| DatabaseError::SecretUnavailable)
    }
}

fn restore_password(template: &str, password: &str) -> Result<String, ()> {
    let mut url = url::Url::parse(template).map_err(|_| ())?;
    let encoded =
        percent_encoding::utf8_percent_encode(password, percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    url.set_password(Some(&encoded)).map_err(|_| ())?;
    Ok(url.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_password_without_changing_other_url_components() {
        let result =
            restore_password("mysql://user:{pass}@[::1]:3306/db?x={pass}", "p#@:%").unwrap();
        let url = url::Url::parse(&result).unwrap();
        assert_eq!(
            percent_encoding::percent_decode_str(url.password().unwrap())
                .decode_utf8()
                .unwrap(),
            "p#@:%"
        );
        assert_eq!(url.query(), Some("x={pass}"));
        assert_eq!(url.path(), "/db");
    }
}
