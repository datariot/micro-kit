//! #Micro-Kit
//!
//! The Micro-Kit module is meant to be a curated collection of crates (Re-Exported) and helper
//! functionality to build `RESTful` micro-services with standardized logging, healthchecking,
//! metrics, and configuration
//!
//! The motivation for the module is from Dropwizard, a collection of java libs that help quickly
//! build standardized `RESTful` applications. This kit is nowhere close to as useful (yet) but we
//! can aspire!

extern crate log4rs;
extern crate metrics as metrics_lib;

pub extern crate yaml_rust as yaml;
pub extern crate iron;
pub extern crate router;

pub extern crate serde;
pub extern crate serde_bytes as bytes;
pub extern crate serde_json as json;

pub extern crate chrono;

/// Configuration based on YAML files for apps.
pub mod config;

/// Healthchecks for apps.
pub mod healthcheck;

/// `RESTful` API helpers.
pub mod http;

/// Logging configuration for apps.
pub mod logging;

/// A framework for adding metrics to an app.
pub mod metrics;

use std::ops::Deref;
use chrono::prelude::{TimeZone, Utc, DateTime};
use serde::{Deserialize, Serialize, Deserializer, Serializer};

/// A `TimeStamp` newtype around the i64 so we can implement
/// serialization to and from Epoch timestamps in a type safe manner.
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy, Hash)]
pub struct TimeStamp(i64);

impl TimeStamp {

    /// Create a new `TimeStamp`. Here we use an i64 because several modern time packages have moved
    /// to an i64 representation of epoch.
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

impl From<DateTime<Utc>> for TimeStamp {
    fn from(dt: DateTime<Utc>) -> Self {
        TimeStamp(dt.timestamp())
    }
}

impl Into<DateTime<Utc>> for TimeStamp {
    fn into(self) -> DateTime<Utc> {
        Utc.timestamp(*self, 0)
    }
}

impl<'de> Deserialize<'de> for TimeStamp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        i64::deserialize(deserializer).map(TimeStamp::new)
    }
}
