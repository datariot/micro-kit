use std::fmt;
use std::error::Error as StdError;
use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::io::prelude::Read;
use std::ops::Index;

use ::yaml::YamlLoader;
use ::yaml::Yaml;
use ::yaml::scanner::ScanError;
use ::yaml::emitter::EmitError;

/// When loading a YAML file there maybe missing required components or bad YAML. This error type
/// allows these errors to be propegated.
#[derive(Debug)]
pub enum ConfigError {
    /// A problem finding or reading the config file.
    IoError(io::Error),
    /// When a yaml file is missing a required component. Will return the missing component.
    MissingComponent(String),
    /// When a YAML file encounters a parsing error.
    YamlSyntax(ScanError),
    /// When there is a problem emitting yaml.
    YamlEmit(EmitError),
    /// Handle a Box<Error>
    Error(Box<StdError>),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::IoError(ref err) => write!(f, "ConfigError: {}", err),
            ConfigError::MissingComponent(ref comp) => write!(f, "ConfigError: Missing {}", comp),
            ConfigError::YamlSyntax(ref err) => write!(f, "ConfigError: {}", err),
            ConfigError::YamlEmit(ref err) => write!(f, "ConfigError: {:?}", err),
            ConfigError::Error(ref err) => write!(f, "ConfigError: {:?}", err),
        }
    }
}

impl StdError for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::IoError(ref err) => err.description(),
            ConfigError::MissingComponent(_) => "Missing Required Component",
            ConfigError::YamlSyntax(ref err) => err.description(),
            ConfigError::YamlEmit(_) => "Problem emitting Yaml",
            ConfigError::Error(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            ConfigError::IoError(ref err) => Some(err),
            ConfigError::MissingComponent(_) => None,
            ConfigError::YamlSyntax(ref err) => Some(err),
            ConfigError::YamlEmit(_) => None,
            ConfigError::Error(_) => None,
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
        ConfigError::YamlSyntax(err)
    }
}

impl From<EmitError> for ConfigError {
    fn from(err: EmitError) -> ConfigError {
        ConfigError::YamlEmit(err)
    }
}

impl From<Box<StdError>> for ConfigError {
    fn from(err: Box<StdError>) -> ConfigError {
        ConfigError::Error(err)
    }
}

/// Wrapper for a parsed configuration. Should wrap the Yaml better to make the API easier to use.
#[derive(Debug, Clone)]
pub struct ConfigFile {
    yaml: Yaml,
}

impl ConfigFile {
    /// Open a yaml file as a config file.
    pub fn open<'a>(name: Cow<'a, str>) -> Result<Self, ConfigError> {
        let mut config_file = File::open(name.as_ref())?;
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).expect("Error reading config file");
        let yamls = YamlLoader::load_from_str(&config_str)?;
        let yaml: Yaml = yamls[0].clone();
        Ok(ConfigFile {
            yaml: yaml
        })
    }
}

static BAD_VALUE: Yaml = Yaml::BadValue;

/// Allow a `ConfigFile` to be accessed like a map
impl<'a> Index<&'a str> for ConfigFile {
    type Output = Yaml;

    fn index(&self, idx: &'a str) -> &Yaml {
        let key = Yaml::String(idx.to_owned());
        match self.yaml.as_hash() {
            Some(h) => h.get(&key).unwrap_or(&BAD_VALUE),
            None => &BAD_VALUE
        }
    }
}

/// Allow a `ConfigFile` to be accessed like an array.
impl Index<usize> for ConfigFile {
    type Output = Yaml;

    fn index(&self, idx: usize) -> &Yaml {
        if let Some(v) = self.yaml.as_vec() {
            v.get(idx).unwrap_or(&BAD_VALUE)
        } else if let Some(v) = self.yaml.as_hash() {
            let key = Yaml::Integer(idx as i64);
            v.get(&key).unwrap_or(&BAD_VALUE)
        } else {
            &BAD_VALUE
        }
    }
}
