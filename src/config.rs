use std::fmt;
use std::error::Error as StdError;
use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::io::prelude::Read;
use ::yaml::YamlLoader;
use ::yaml::Yaml;
use ::yaml::scanner::ScanError;

/// When loading a YAML file there maybe missing required components or bad YAML. This error type
/// allows these errors to be propegated.
#[derive(Debug)]
pub enum ConfigError {
    /// A problem finding or reading the config file.
    IoError(io::Error),
    /// When a yaml file is missing a required component. Will return the missing component.
    MissingComponent(String),
    /// When a YAML file encounters a parsing error.
    BadYaml(ScanError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::IoError(ref err) => write!(f, "ConfigError: {}", err),
            ConfigError::MissingComponent(ref comp) => write!(f, "ConfigError: Missing {}", comp),
            ConfigError::BadYaml(ref err) => write!(f, "ConfigError: {}", err),
        }
    }
}

impl StdError for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::IoError(ref err) => err.description(),
            ConfigError::MissingComponent(_) => "Missing Required Component",
            ConfigError::BadYaml(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            ConfigError::IoError(ref err) => Some(err),
            ConfigError::MissingComponent(_) => None,
            ConfigError::BadYaml(ref err) => Some(err)
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::IoError(err)
    }
}

impl From<ScanError> for ConfigError {
    fn from(err: ScanError) -> ConfigError {
        ConfigError::BadYaml(err)
    }
}

/// Wrapper for a parsed configuration. Should wrap the Yaml better to make the API easier to use.
pub struct ConfigFile {
    yaml: Yaml,
}

impl ConfigFile {

    pub fn open<'a>(name: Cow<'a, str>) -> Result<Self, ConfigError> {
        let mut config_file = File::open(name.as_ref())?;
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).expect("Error reading config file");
        let yaml = YamlLoader::load_from_str(&config_str)?;
        Ok(ConfigFile {
            yaml: yaml[0].clone()
        })
    }

    pub fn get_config(&self) -> &Yaml {
        &self.yaml
    }

}

pub struct APIConfig<'a> {
    addr: Cow<'a, str>,
    port: u16
}

impl<'a> APIConfig<'a> {
    pub fn new(c: &'a ConfigFile) -> Result<Self, ConfigError> {
        if !c.get_config()["service"].is_badvalue() {
            if !c.get_config()["service"]["address"].is_badvalue() {
                let service_ip = c.get_config()["service"]["address"].as_str().unwrap();
                let service_port = c.get_config()["service"]["port"].as_i64().unwrap_or(8081) as u16;

                Ok(APIConfig {
                    addr: service_ip.into(),
                    port: service_port
                })
            } else {
                Err(ConfigError::MissingComponent("service -> address".to_string()))
            }

        } else {
            Err(ConfigError::MissingComponent("service".to_string()))
        }
    }

    pub fn get_conn(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}
