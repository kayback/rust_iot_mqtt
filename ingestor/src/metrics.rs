use lazy_static::lazy_static;
use prometheus::{Counter, Encoder, Gauge, Histogram, HistogramOpts, Opts, Registry, TextEncoder};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref MESSAGES_TOTAL: Counter = Counter::with_opts(Opts::new(
        "ingestor_messages_total",
        "Total messages received from MQTT"
    ))
    .unwrap();
    pub static ref VALID_MESSAGES_TOTAL: Counter = Counter::with_opts(Opts::new(
        "ingestor_valid_messages_total",
        "Total valid messages after validation"
    ))
    .unwrap();
    pub static ref INVALID_MESSAGES_TOTAL: Counter = Counter::with_opts(Opts::new(
        "ingestor_invalid_messages_total",
        "Total invalid messages rejected"
    ))
    .unwrap();
    pub static ref DB_FAILURES_TOTAL: Counter = Counter::with_opts(Opts::new(
        "ingestor_db_failures_total",
        "Total database insert failures"
    ))
    .unwrap();
    pub static ref INGEST_LATENCY_SECONDS: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "ingestor_ingest_latency_seconds",
            "Time taken to ingest batch into DB"
        )
        .buckets(vec![
            0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0
        ])
    )
    .unwrap();
    pub static ref BATCH_SIZE: Gauge = Gauge::with_opts(Opts::new(
        "ingestor_batch_size",
        "Current batch size being processed"
    ))
    .unwrap();
    pub static ref CHANNEL_FULL_TOTAL: Counter = Counter::with_opts(Opts::new(
        "ingestor_channel_full_total",
        "Total number of times channel was full (backpressure events)"
    ))
    .unwrap();
}

pub fn init_metrics() {
    REGISTRY.register(Box::new(MESSAGES_TOTAL.clone())).unwrap();
    REGISTRY
        .register(Box::new(VALID_MESSAGES_TOTAL.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(INVALID_MESSAGES_TOTAL.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(DB_FAILURES_TOTAL.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(INGEST_LATENCY_SECONDS.clone()))
        .unwrap();
    REGISTRY.register(Box::new(BATCH_SIZE.clone())).unwrap();
    REGISTRY
        .register(Box::new(CHANNEL_FULL_TOTAL.clone()))
        .unwrap();
}

pub fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
