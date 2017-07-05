use std::sync::{PoisonError, MutexGuard};
use std::fmt;
use std::error::Error;
use std::borrow::Cow;
use std::any::Any;

use ::json;
use ::config::{ConfigFile, ConfigError};

#[derive(Debug)]
pub enum APIError<'a> {
    PoisonError(PoisonError<MutexGuard<'a, Any>>),
    JsonError(json::Error)
}

impl<'a> fmt::Display for APIError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            APIError::PoisonError(_) => f.write_str("Lock Error"),
            APIError::JsonError(_) => f.write_str("Json encoding Error")
        }
    }
}

impl<'a> Error for APIError<'a> {
    fn description(&self) -> &str {
        match *self {
            APIError::PoisonError(_) => "Lock Error",
            APIError::JsonError(_) => "JsonError"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            APIError::PoisonError(ref e) => Some(e),
            APIError::JsonError(ref e) => Some(e)
        }
    }
}

pub struct APIConfig<'a> {
    addr: Cow<'a, str>,
    port: u16
}

impl<'a> APIConfig<'a> {
    pub fn new(c: &'a ConfigFile) -> Result<Self, ConfigError> {
        if !c["service"].is_badvalue() {
            if !c["service"]["address"].is_badvalue() {
                let service_ip = c["service"]["address"].as_str().unwrap();
                let service_port = c["service"]["port"].as_i64().unwrap_or(8081) as u16;

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
