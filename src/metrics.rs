pub use metrics_lib::metrics::{Metric, Counter, StdCounter};

use std::collections::HashMap;

use ::json;

use ::iron::prelude::{IronResult, Response};
use ::iron::status;

pub struct MetricsService {

    metrics: HashMap<String, Metric>,
    reporter_name: String,

}

impl MetricsService {

    pub fn get_name(&self) -> &str {
        &(*self.reporter_name)
    }

    pub fn add<S: Into<String>>(&mut self, name: S, metric: Metric) -> Result<(), String> {
        let n = name.into();
        match self.metrics.insert(n.clone(), metric) {
            Some(_) => Ok(()),
            None => Err(format!("Unable to attach metric reporter {}, {:?}", n, self.metrics.len())),
        }
    }

    pub fn new<S: Into<String>>(reporter_name: S) -> Self {
        MetricsService {
            metrics: HashMap::new(),
            reporter_name: reporter_name.into(),
        }
    }

    pub fn report(&self) -> IronResult<Response> {
        let mut report: HashMap<&String,i64> = HashMap::new();
        for (name, metric) in &self.metrics {
            let snapshot = match *metric {
                Metric::Meter(ref x) => {
                    x.snapshot().count as i64
                }
                Metric::Gauge(ref x ) => {
                    x.snapshot().value as i64
                }
                Metric::Counter(ref x) => {
                    x.snapshot().value as i64
                }
                Metric::Histogram(_) => {
                   -1 as i64
                }
            };
            report.insert(name, snapshot);
        }
        match json::to_string(&report) {
            Ok(r) => {
                Ok(Response::with((status::Ok, r)))
            },
            Err(e) => Ok(Response::with((status::InternalServerError, format!("{:?}", e))))
        }

    }
}
