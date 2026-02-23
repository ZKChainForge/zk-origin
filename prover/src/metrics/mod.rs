// prover/src/metrics/mod.rs

use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge,
    Registry, TextEncoder, Encoder,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref PROOFS_GENERATED: IntCounter = IntCounter::new(
        "zkorigin_proofs_generated_total",
        "Total number of proofs generated"
    ).unwrap();
    
    pub static ref PROOFS_VERIFIED: IntCounter = IntCounter::new(
        "zkorigin_proofs_verified_total",
        "Total number of proofs verified"
    ).unwrap();
    
    pub static ref PROOF_GENERATION_TIME: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "zkorigin_proof_generation_seconds",
            "Time to generate a proof"
        ).buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
    ).unwrap();
    
    pub static ref PROOF_VERIFICATION_TIME: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "zkorigin_proof_verification_seconds",
            "Time to verify a proof"
        ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1])
    ).unwrap();
    
    pub static ref CURRENT_LINEAGE_DEPTH: IntGauge = IntGauge::new(
        "zkorigin_current_lineage_depth",
        "Current maximum lineage depth"
    ).unwrap();
    
    pub static ref POLICY_VIOLATIONS: IntCounter = IntCounter::new(
        "zkorigin_policy_violations_total",
        "Number of rejected transitions due to policy"
    ).unwrap();
    
    pub static ref ACTIVE_PROVERS: IntGauge = IntGauge::new(
        "zkorigin_active_provers",
        "Number of active prover instances"
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(PROOFS_GENERATED.clone())).unwrap();
    REGISTRY.register(Box::new(PROOFS_VERIFIED.clone())).unwrap();
    REGISTRY.register(Box::new(PROOF_GENERATION_TIME.clone())).unwrap();
    REGISTRY.register(Box::new(PROOF_VERIFICATION_TIME.clone())).unwrap();
    REGISTRY.register(Box::new(CURRENT_LINEAGE_DEPTH.clone())).unwrap();
    REGISTRY.register(Box::new(POLICY_VIOLATIONS.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_PROVERS.clone())).unwrap();
}

pub fn export_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

// Metrics endpoint handler
pub async fn metrics_handler() -> impl warp::Reply {
    warp::reply::with_header(
        export_metrics(),
        "Content-Type",
        "text/plain; charset=utf-8"
    )
}