use config;

/// config utils
pub struct ConfigUtils {
    pub settings: config::Config,
}

impl ConfigUtils {
    pub fn init(fpath: &str) -> Self {
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name(fpath)).unwrap();
        Self { settings }
    }

    pub fn get(&self, key: &str) -> Result<String, config::ConfigError> {
        self.settings.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // config.ini content
    // debug = false
    // port = 3223
    // host = "0.0.0.0"
    #[test]
    fn test_config_init() {
        let config = ConfigUtils::init("config.ini");
        println!("{}", config.get("port").unwrap());
    }
}
