extern crate log4rs;
extern crate metrics as metrics_lib;

pub extern crate yaml_rust as yaml;
pub extern crate iron as http;

pub extern crate serde;
pub extern crate serde_bytes as bytes;
pub extern crate serde_json as json;

pub extern crate rmp as msgpack;
pub extern crate rmp_serde as msgpack_serde;

pub extern crate chrono;

pub mod healthcheck;
pub mod logging;
pub mod metrics;
pub mod config;

use std::ops::Deref;
use chrono::offset::TimeZone;
use chrono::offset::utc::UTC;
use chrono::datetime::DateTime;
use serde::{Deserialize, Serialize, Deserializer, Serializer};

// A TimeStamp new type around the i64 so we can implement
// Serialization to and from Epoch timestamps in a type safe manner.

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy, Hash)]
pub struct TimeStamp(i64);

impl TimeStamp {
    pub fn new(ts: i64) -> TimeStamp {
        TimeStamp(ts)
    }
}

impl Deref for TimeStamp {
    type Target = i64;

    fn deref(&self) -> &i64 {
        let &TimeStamp(ref value) = self;
        value
    }
}

impl Serialize for TimeStamp {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where S: Serializer
    {
        serializer.serialize_i64(**self)
    }
}

impl From<DateTime<UTC>> for TimeStamp {
    fn from(dt: DateTime<UTC>) -> Self {
        TimeStamp(dt.timestamp())
    }
}

impl Into<DateTime<UTC>> for TimeStamp {
    fn into(self) -> DateTime<UTC> {
        UTC.timestamp(*self, 0)
    }
}

impl<'de> Deserialize<'de> for TimeStamp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        i64::deserialize(deserializer).map(|b| TimeStamp::new(b))
    }
}
