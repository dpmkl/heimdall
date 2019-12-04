use failure::Fail;
use serde::{Deserialize, Serialize};
use std::io;
use std::net::SocketAddr;
use toml;

#[derive(Serialize, Deserialize, Debug)]
pub struct RouteDefinition {
    pub source: String,
    pub target: SocketAddr,
    pub target_path: Option<String>,
    pub allowed_methods: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub listen: SocketAddr,
    pub cert_file: String,
    pub cert_pass: Option<String>,
    pub routes: Vec<RouteDefinition>,
}

impl Default for Config {
    fn default() -> Self {
        let methods: Vec<String> = vec!["GET".to_owned(), "POST".to_owned()];
        let mut routes: Vec<RouteDefinition> = Vec::new();
        routes.push(RouteDefinition {
            source: "/".to_string(),
            target: "127.0.0.1:8000".parse().unwrap(),
            target_path: None,
            allowed_methods: vec![],
        });
        routes.push(RouteDefinition {
            source: "/stuff".to_string(),
            target: "127.0.0.1:7000".parse().unwrap(),
            target_path: None,
            allowed_methods: methods,
        });
        Self {
            listen: "0.0.0.0:8443".parse().unwrap(),
            cert_file: "identity.p12".to_owned(),
            cert_pass: Some("mypass".to_owned()),
            routes,
        }
    }
}

pub fn load(file: &str) -> Result<Config, ConfigError> {
    let data = std::fs::read_to_string(file)?;
    Ok(toml::from_str(&data)?)
}

pub fn write_default(file: &str) -> Result<(), ConfigError> {
    let config = Config::default();
    let data = toml::to_string_pretty(&config)?;
    std::fs::write(file, data)?;
    Ok(())
}

#[derive(Debug, Fail)]
pub enum ConfigError {
    #[fail(display = "Io error: {}", err)]
    Io { err: io::Error },
    #[fail(display = "Toml error: {:?}", err)]
    TomlDe { err: toml::de::Error },
    #[fail(display = "Toml error: {:?}", err)]
    TomlSer { err: toml::ser::Error },
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::Io { err }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> ConfigError {
        ConfigError::TomlDe { err }
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> ConfigError {
        ConfigError::TomlSer { err }
    }
}
