#![allow(dead_code)]
use std::collections::HashMap;

use crate::*;

/// Contains a time as u64 seconds.
#[derive(Default, Debug)]
pub struct Time(pub u64);

/// Contains a path on the filesystem.
#[derive(Default, Debug)]
pub struct Filesystem {
    pub path: &'static str,
}

/// a HashMap of metrics.
pub struct Metrics(pub HashMap<String, u64>);

/// Builds a hashmap containing some example metrics.
pub fn build_initial_metrics() -> Metrics {
    let mut h = HashMap::new();
    h.insert("successes".into(), 5);
    h.insert("failures".into(), 10);
    Metrics(h)
}

decl_fixed_typemap!(
    pub struct ExampleMap {
        /// Let's let the name field be public.
        name: String,
        _: Time,
        _: Filesystem,
        _: Metrics = build_initial_metrics(),
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getters() {
        let mut map = ExampleMap::new();

        map.get_mut::<Time>().0 = 5;
        assert_eq!(map.get::<Time>().0, 5);

        assert_eq!(map.get::<Filesystem>().path, "");
        map.get_mut::<Filesystem>().path = "foo";
        assert_eq!(map.get::<Filesystem>().path, "foo");

        {
            let metrics = &map.get::<Metrics>().0;
            assert_eq!(metrics.get("successes"), Some(&5));
            assert_eq!(metrics.get("failures"), Some(&10));
        }

        {
            let metrics_mut = &mut map.get_mut::<Metrics>().0;
            metrics_mut.insert("bar".into(), 15);
            assert_eq!(metrics_mut.get("bar"), Some(&15));
        }
    }
}
