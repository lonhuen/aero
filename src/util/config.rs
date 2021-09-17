use std::net::Ipv4Addr;
use std::str::FromStr;

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

    pub fn get(&self, key: &str) -> String {
        self.settings.get(key).unwrap()
    }

    pub fn get_addr(&self, key: &str) -> Ipv4Addr {
        let addr: String = self.settings.get(key).unwrap();
        Ipv4Addr::from_str(&addr).unwrap()
    }

    pub fn get_int(&self, key: &str) -> i64 {
        self.settings.get_int(key).unwrap()
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
        println!("{}", config.get("port"));
    }
}
