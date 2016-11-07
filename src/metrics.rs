use metrics_lib::metrics::Metric;

use std::collections::HashMap;

use serde_json;

use iron::prelude::*;
use iron::status;

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
            None => Err(format!("Unable to attach metric reporter {}", n)),
        }
    }

    pub fn new<S: Into<String>>(reporter_name: S) -> Self {
        MetricsService {
            metrics: HashMap::new(),
            reporter_name: reporter_name.into(),
        }
    }

    pub fn report(&self) -> IronResult<Response> {
        let mut report: HashMap<&String,String> = HashMap::new();
        for (name, metric) in &self.metrics {
            let snapshot = match metric {
                &Metric::Meter(ref x) => {
                    format!("{:?}", x.snapshot())
                }
                &Metric::Gauge(ref x) => {
                    format!("{:?}", x.snapshot())
                }
                &Metric::Counter(ref x) => {
                    format!("{:?}", x.snapshot())
                }
                &Metric::Histogram(ref x) => {
                    format!("histogram{:?}", x)
                }
            };
            report.insert(name, snapshot);
        }
        match serde_json::to_string(&report) {
            Ok(r) => {
                println!("{:?}", r);
                Ok(Response::with((status::Ok, r)))
            },
            Err(e) => Ok(Response::with((status::InternalServerError, format!("{:?}", e))))
        }
        
    }
}

