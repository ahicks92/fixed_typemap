#![allow(dead_code)]
use std::collections::HashMap;

use crate::*;

/// Contains a time as u64 seconds.
pub struct Time(pub u64);

/// Contains a path on the filesystem.
pub struct Filesystem {
    pub path: &'static str,
}

/// Refers to a HashMap of metrics.
pub struct MetricsKey;

/// Builds a hashmap containing some example metrics.
pub fn build_initial_metricss() -> HashMap<String, u64> {
    let mut h = HashMap::new();
    h.insert("successes".into(), 5);
    h.insert("failures".into(), 10);
    h
}

decl_fixed_typemap!(
    pub struct ExampleMap {
        /// Let's let the name field be public.
        name: String,
        _: Time,
        _: Filesystem,
        _: MetricsKey -> HashMap<String, u64> = build_initial_metrics(),
    }
);
