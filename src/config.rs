use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::str::FromStr;
use std::fmt;
use std::error::Error as StdError;
use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::Read;
use yaml_rust::YamlLoader;
use yaml_rust::Yaml;

use ::kafka::config::{ClientConfig, TopicConfig};
use ::kafka::client::EmptyContext;
use ::kafka::error::KafkaError;
use ::kafka::producer::FutureProducer;
use ::kafka::consumer::{BaseConsumer, EmptyConsumerContext};

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

pub struct KafkaProducerConfig<'a> {
    env: Cow<'a, str>,
    client:  Cow<'a, str>,
    topic:  Cow<'a, str>,
    brokers: Vec<String>,
}

impl<'a> KafkaProducerConfig<'a> {

    pub fn new(c: &'a Yaml) -> Result<Self, ConfigError> {
        if !c["kafka"].is_badvalue() && !c["kafka"]["producer"].is_badvalue(){
            let env = c["kafka"]["env"].as_str().expect("No kafka environment (env) found");
            let client = c["kafka"]["client"].as_str().expect("No kafka client id (client) found");
            let pro_topic = c["kafka"]["producer"]["topic"].as_str().expect("No kafka producer topic found");
            let brokers: Vec<String> = c["kafka"]["brokers"].as_vec().unwrap()
                .iter().map(|v| v.as_str().unwrap().to_string()).collect();

            Ok(KafkaProducerConfig {
                brokers: brokers,
                topic: pro_topic.into(),
                client: client.into(),
                env: env.into(),
            })
        } else {
            Err(ConfigError::MissingComponent("kafka".to_string()))
        }
    }

    pub fn create(&self) -> Result<FutureProducer<EmptyContext>, KafkaError> {
        ClientConfig::new()
            .set("bootstrap.servers", self.brokers.join(",").as_str())
            .set("compression.codec", "snappy")
            .create::<FutureProducer<EmptyContext>>()
    }

    pub fn get_topic(&self) -> &str {
        self.topic.as_ref()
    }

    pub fn get_env(&self) -> &str {
        self.env.as_ref()
    }

    pub fn get_client(&self) -> &str {
        self.client.as_ref()
    }

}

pub struct KafkaConsumerConfig<'a> {
    env: Cow<'a, str>,
    client:  Cow<'a, str>,
    topic: Cow<'a, str>,
    group: Cow<'a, str>,
    topic_start: Cow<'a, str>,
    max_fetch: Cow<'a, str>,
    brokers: Vec<String>,
}

impl<'a> KafkaConsumerConfig<'a> {

    pub fn new(c: &'a Yaml) -> Result<Self, ConfigError> {
        if !c["kafka"].is_badvalue() {
            let env = c["kafka"]["env"].as_str().expect("No kafka environment (env) found");
            let client = c["kafka"]["client"].as_str().expect("No kafka client id (client) found");
            let topic = c["kafka"]["consumer"]["topic"].as_str().expect("No kafka consumer topic found");
            let group = c["kafka"]["consumer"]["group"].as_str().expect("No kafka consumer group found");
            let max_fetch = c["kafka"]["consumer"]["max_fetch"].as_str().unwrap_or("1048576");
            let topic_start = c["kafka"]["consumer"]["start"].as_str().unwrap_or("smallest");
            let brokers: Vec<String> = c["kafka"]["brokers"].as_vec()
                .expect("no kafka brokers (brokers) found")
                .iter().map(|v| v.as_str().unwrap().to_string()).collect();

            Ok(KafkaConsumerConfig {
                env: env.into(),
                client: client.into(),
                brokers: brokers,
                topic: topic.into(),
                group: group.into(),
                topic_start: topic_start.into(),
                max_fetch: max_fetch.into(),
            })
        } else {
            Err(ConfigError::MissingComponent("kafka".to_string()))
        }
    }

    pub fn create(&self) -> Result<BaseConsumer<EmptyConsumerContext>, KafkaError> {
        ClientConfig::new()
            .set("group.id", self.group.as_ref())
            .set("bootstrap.servers", self.brokers.join(",").as_str())
            .set("max.partition.fetch.bytes", self.max_fetch.as_ref())
            .set_default_topic_config(
                 TopicConfig::new()
                 .set("auto.offset.reset", self.topic_start.as_ref())
                 .finalize())
            .create::<BaseConsumer<_>>()
    }

    pub fn get_topic(&self) -> &str {
        self.topic.as_ref()
    }

    pub fn get_client(&self) -> &str {
        self.client.as_ref()
    }

    pub fn get_env(&self) -> &str {
        self.env.as_ref()
    }
}
