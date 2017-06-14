use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::str::FromStr;
use std::fmt;
use std::error::Error as StdError;
use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::Read;
use ::yaml::YamlLoader;
use ::yaml::Yaml;

#[derive(Debug)]
pub enum ConfigError {
    MissingComponent(String)
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::MissingComponent(_) => f.write_str("MissingConfig")
        }
    }
}

impl StdError for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::MissingComponent(_) => "Missing Required Component"
        }
    }
}

pub struct ConfigFile;

impl ConfigFile {

    pub fn open(name: &str) -> Yaml {
        let mut config_file = File::open(name).expect("Error reading config file");
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).expect("Error reading config file");
        match YamlLoader::load_from_str(&config_str) {
            Err(e) => panic!("Error parsing YAML: {:?}", e),
            Ok(c) => c[0].clone(),
        }
    }

}

pub struct APIConfig<'a> {
    addr: Cow<'a, str>,
    port: u16
}

impl<'a> APIConfig<'a> {
    pub fn new(c: &'a Yaml) -> Result<Self, ConfigError> {
        if !c["service"].is_badvalue() {
            let service_ip = c["service"]["address"].as_str().unwrap_or("0.0.0.0");
            let service_port = c["service"]["port"].as_i64().unwrap_or(8081) as u16;

            Ok(APIConfig {
                addr: service_ip.into(),
                port: service_port
            })
        } else {
            Err(ConfigError::MissingComponent("netflow".to_string()))
        }
    }

    pub fn get_conn(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}

pub struct NetflowSocketConfig {
    socket: SocketAddr
}

impl NetflowSocketConfig {
    pub fn new(c: &Yaml) -> Result<Self, ConfigError> {
        if !c["netflow"].is_badvalue() {
            let netflow_ip = c["netflow"]["address"].as_str().unwrap_or("0.0.0.0");
            let netflow_port = c["netflow"]["port"].as_i64().unwrap_or(9995) as u16;

            let ip = Ipv4Addr::from_str(netflow_ip).unwrap();
            Ok(NetflowSocketConfig {
                socket: SocketAddr::V4(SocketAddrV4::new(ip, netflow_port))
            })
        } else {
            Err(ConfigError::MissingComponent("netflow".to_string()))
        }
    }

    pub fn get_socket(&self) -> &SocketAddr {
        &self.socket
    }
}
