use ::yaml::{Yaml, YamlEmitter};
use log4rs;

pub use log4rs::init_config;

#[derive(Debug)]
pub struct LoggingConfig;

impl LoggingConfig {
    pub fn config_logging(config_yaml: &Yaml) -> log4rs::config::Config {
        let mut log_config_buf = String::new();
        {
            let mut log_emitter = YamlEmitter::new(&mut log_config_buf);
            let _ = log_emitter.dump(&config_yaml["logging"]).unwrap();
        }
        let log_config_str = log_config_buf.as_str();
        let log_config = log4rs::file::Config::parse(log_config_str,
                                                     log4rs::file::Format::Yaml,
                                                     &log4rs::file::Deserializers::default())
            .unwrap();
        log_config.into_config()
    }
}
