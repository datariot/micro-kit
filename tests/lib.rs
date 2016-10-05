extern crate micro_kit;

use micro_kit::healthcheck::*;

#[test]
fn test_status_and() {

    assert_eq!(HealthCheckStatus::Healthy, HealthCheckStatus::Healthy & HealthCheckStatus::Healthy);
    assert_eq!(HealthCheckStatus::Unhealthy, HealthCheckStatus::Unhealthy & HealthCheckStatus::Healthy);  
    assert_eq!(HealthCheckStatus::Unhealthy, HealthCheckStatus::Unhealthy & HealthCheckStatus::Unhealthy); 
    assert_eq!(HealthCheckStatus::Unhealthy, HealthCheckStatus::Healthy & HealthCheckStatus::Unhealthy);
}

#[test]
fn test_health_check() {

    #[derive(Debug)]
    struct GoodHealthCheck;
    impl HealthCheck for GoodHealthCheck {
        fn name(&self) -> String {
            "Good".to_string()
        }

        fn check_health(&mut self) -> HealthCheckStatus {
            HealthCheckStatus::Healthy
        }
    }

    struct BadHealthCheck;
    impl HealthCheck for BadHealthCheck {
        fn name(&self) -> String {
            "Bad".to_string()
        }

        fn check_health(&mut self) -> HealthCheckStatus {
            HealthCheckStatus::Unhealthy
        }
    }

    let good = GoodHealthCheck{};
    let bad = BadHealthCheck{};

    let mut service = HealthCheckService::new();
    service.register_check(good);

    let (status, checks) = service.execute();
    assert_eq!(1, checks.len());
    assert_eq!(Some(&HealthCheckStatus::Healthy), checks.get(&"Good".to_string()));
    assert_eq!(HealthCheckStatus::Healthy, status);

    service.register_check(bad);

    let (status, checks) = service.execute();
    assert_eq!(2, checks.len());
    assert_eq!(Some(&HealthCheckStatus::Healthy), checks.get(&"Good".to_string()));
    assert_eq!(Some(&HealthCheckStatus::Unhealthy), checks.get(&"Bad".to_string()));

    assert_eq!(HealthCheckStatus::Unhealthy, status);

}