use std::ops::BitAnd;
use std::collections::HashMap;

use iron::prelude::*;
use iron::status;

use serde::{Serialize, Serializer};
use serde_json;

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
            _ => false,
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

impl Serialize for HealthCheckStatus {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where S: Serializer
    {
        match self {
            &HealthCheckStatus::Healthy => serializer.serialize_str("Ok"),
            &HealthCheckStatus::Unhealthy => serializer.serialize_str("Failed"),
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
    checks: Vec<Box<HealthCheck + 'static>>,
}

impl HealthCheckService {

    pub fn new() -> HealthCheckService {
        HealthCheckService { checks: Vec::new() }
    }

    pub fn register_check<H: HealthCheck + 'static>(&mut self, check: H) {
        self.checks.push(Box::new(check));
    }

    pub fn check_service_health(&mut self, _: &mut Request) -> IronResult<Response> {
        let (global, health) = self.execute();

        let payload = serde_json::to_string(&health).unwrap();
        let status: status::Status = global.into();

        Ok(Response::with((status, payload)))
    }

    pub fn execute(&mut self) -> (HealthCheckStatus, HashMap<String, HealthCheckStatus>) {
        let mut map = HashMap::new();

        for check in &mut self.checks {
            let res = check.check_health();
            map.insert(check.name(), res);
        }

        let global_health = map.values()
            .fold(HealthCheckStatus::Healthy, |check, val| check & val.clone());

        (global_health, map)
    }
}
