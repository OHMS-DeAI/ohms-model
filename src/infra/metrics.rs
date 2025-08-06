use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Metrics {
    pub total_models: u64,
    pub active_models: u64,
    pub pending_models: u64,
    pub deprecated_models: u64,
    pub total_chunks: u64,
    pub total_chunk_accesses: u64,
    pub upload_requests: u64,
    pub activation_requests: u64,
    pub errors: HashMap<String, u64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_models: 0,
            active_models: 0,
            pending_models: 0,
            deprecated_models: 0,
            total_chunks: 0,
            total_chunk_accesses: 0,
            upload_requests: 0,
            activation_requests: 0,
            errors: HashMap::new(),
        }
    }
}

thread_local! {
    static METRICS: std::cell::RefCell<Metrics> = std::cell::RefCell::new(Metrics::default());
}

pub fn increment_counter(counter: &str) {
    METRICS.with(|metrics| {
        let mut m = metrics.borrow_mut();
        match counter {
            "upload_requests" => m.upload_requests += 1,
            "activation_requests" => m.activation_requests += 1,
            "chunk_accesses" => m.total_chunk_accesses += 1,
            _ => {}
        }
    });
}

pub fn increment_error(error_type: &str) {
    METRICS.with(|metrics| {
        let mut m = metrics.borrow_mut();
        let current = m.errors.get(error_type).copied().unwrap_or(0);
        m.errors.insert(error_type.to_string(), current + 1);
    });
}

pub fn update_model_counts(active: u64, pending: u64, deprecated: u64) {
    METRICS.with(|metrics| {
        let mut m = metrics.borrow_mut();
        m.active_models = active;
        m.pending_models = pending;
        m.deprecated_models = deprecated;
        m.total_models = active + pending + deprecated;
    });
}

pub fn get_metrics() -> Metrics {
    METRICS.with(|metrics| metrics.borrow().clone())
}