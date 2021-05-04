use std::env;

use flexi_logger::Logger;
use log::info;
use serde::Deserialize;
use warp::{http::StatusCode, Filter};

#[tokio::main]
async fn main() {
    run().await
}

async fn run() {
    Logger::with_env_or_str("info").start().unwrap();
    let port = env::var("JLS_EXPORTER_PORT")
        .map_or(Ok(9823), |s| s.parse())
        .expect("failed to parse port as number");
    let jls_stats_token =
        env::var("JLS_STATS_TOKEN").expect("Environment Variable JLS_STATS_TOKEN not set");
    let jls_base_url = env::var("JLS_BASE_URL").expect("Environment Variable JLS_BASE_URL not set");

    let jls_url = format!(
        "{}/licenses-report.json?token={}",
        jls_base_url, jls_stats_token
    );
    let jls_url = Box::leak(jls_url.into_boxed_str()) as &'static str;
    info!("JLS url is {}", jls_url);

    let index = warp::path::end().map(|| "Jetbrains FLS Exporter \n Metrics exported on /metrics");
    let metrics = warp::path("metrics")
        .and(warp::path::end())
        .and_then(move || metrics_handle(jls_url));
    warp::serve(index.or(metrics))
        .run(([127, 0, 0, 1], port))
        .await
}

async fn metrics_handle(jls_url: &str) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    Ok(match metrics(jls_url).await {
        Ok(s) => Box::new(s),
        Err(e) => Box::new(warp::reply::with_status(
            format!(
                "An error occured while trying to contact the license server: \n{}",
                e
            ),
            StatusCode::SERVICE_UNAVAILABLE,
        )),
    })
}

#[derive(Debug, Deserialize)]
struct LicensesReport {
    licenses: Vec<License>,
}
#[derive(Debug, Deserialize)]
struct License {
    name: String,
    available: i64,
    allocated: i64,
}

async fn metrics(jls_url: &str) -> anyhow::Result<String> {
    use prometheus::{Encoder, IntGaugeVec, Opts, Registry, TextEncoder};

    let report: LicensesReport = reqwest::get(jls_url).await?.json().await?;
    let alloc_opts = Opts::new(
        "jls_licenses_allocated",
        "Number of JLS Licenses currently allocated",
    );
    let avail_opts = Opts::new(
        "jls_licenses_available",
        "Number of JLS Licenses currently available",
    );
    let alloc_gauge = IntGaugeVec::new(alloc_opts, &["license_name"])?;
    let avail_gauge = IntGaugeVec::new(avail_opts, &["license_name"])?;

    // Create a Registry and register Counter.
    let r = Registry::new();
    r.register(Box::new(alloc_gauge.clone())).unwrap();
    r.register(Box::new(avail_gauge.clone())).unwrap();

    for license in report.licenses.iter() {
        alloc_gauge
            .with_label_values(&[&license.name])
            .set(license.allocated);
        avail_gauge
            .with_label_values(&[&license.name])
            .set(license.available);
    }
    // Gather the metrics.
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = r.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(String::from_utf8(buffer).unwrap())
}
