use ::yaml::{YamlEmitter};
use ::config::{ConfigFile, ConfigError};
use log4rs;

pub use log4rs::init_config;

#[derive(Debug)]
pub struct LoggingConfig;

impl LoggingConfig {

    pub fn config_logging(config: &ConfigFile) -> Result<log4rs::config::Config, ConfigError> {
        if !config.get_config()["logging"].is_badvalue() {
            let mut log_config_buf = String::new();
            { //Block because of the mutable borrow, keep the mutability in this scope.
                let mut log_emitter = YamlEmitter::new(&mut log_config_buf);
                let _ = log_emitter.dump(&config.get_config()["logging"])?;
            }
            let log_config_str = log_config_buf.as_str();
            let log_config = log4rs::file::Config::parse(log_config_str,
                                                         log4rs::file::Format::Yaml,
                                                         &log4rs::file::Deserializers::default())?;
            Ok(log_config.into_config())
        } else {
            Err(ConfigError::MissingComponent("logging".to_string()))
        }
    }
}
