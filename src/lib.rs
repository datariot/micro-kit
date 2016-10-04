extern crate rustc_serialize;
extern crate iron;

use std::ops::BitAnd;
use std::collections::HashMap;

use iron::prelude::*;
use iron::status;

use rustc_serialize::{Encodable, Encoder};
use rustc_serialize::json;

#[derive(Debug, Clone)]
pub enum HealthCheckStatus {
    Healthy,
    Unhealthy,
}

impl PartialEq for HealthCheckStatus {
    fn eq(&self, other: &HealthCheckStatus) -> bool {
        match (self, other) {
            (&HealthCheckStatus::Healthy, &HealthCheckStatus::Healthy) => true,
            (&HealthCheckStatus::Unhealthy, &HealthCheckStatus::Unhealthy) => true,
            _ => false
        }
    }
}

impl Into<status::Status> for HealthCheckStatus {
    fn into(self) -> status::Status {
        match self {
            HealthCheckStatus::Healthy => status::Ok,
            HealthCheckStatus::Unhealthy => status::InternalServerError,
        }
    }
}

impl Encodable for HealthCheckStatus {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match self {
            &HealthCheckStatus::Healthy => "Ok".encode(s),
            &HealthCheckStatus::Unhealthy => "Failed".encode(s)
        }
    }
}

impl BitAnd for HealthCheckStatus {
    type Output = HealthCheckStatus;

    fn bitand(self, other: HealthCheckStatus) -> HealthCheckStatus {
        match (self, other) {
            (HealthCheckStatus::Healthy, HealthCheckStatus::Healthy) => HealthCheckStatus::Healthy,
            _ => HealthCheckStatus::Unhealthy,
        }
    }
}

pub trait HealthCheck: Send {

    fn name(&self) -> String;

    fn check_health(&mut self) -> HealthCheckStatus;

}

pub struct HealthCheckService {
    checks: Vec<Box<HealthCheck + 'static>>
}

impl HealthCheckService {

    pub fn new() -> HealthCheckService {
        HealthCheckService { checks: Vec::new()}
    }

    pub fn register_check<H: HealthCheck + 'static>(&mut self, check: H) {
        self.checks.push(Box::new(check));
    }

    pub fn check_service_health(&mut self, _: &mut Request) -> IronResult<Response> {
        let (global, health) = self.execute();

        let payload = json::encode(&health).unwrap();
        let status: status::Status = global.into();

        Ok(Response::with((status, payload)))
    }

    pub fn execute(&mut self) -> (HealthCheckStatus, HashMap<String,HealthCheckStatus>) {
        let mut map = HashMap::new();

        for check in &mut self.checks {
            let res = check.check_health();
            map.insert(check.name(), res);
        }

        let global_health = map.values().fold(HealthCheckStatus::Healthy, | check, val | {
            check & val.clone()
        });

        (global_health, map)
    }

}
