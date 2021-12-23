#![allow(dead_code)]
use std::collections::HashMap;

use crate::*;

/// Contains a time as u64 seconds.
#[derive(Default, Debug, Eq, PartialEq)]
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
    pub struct ExampleMapNoStd {
        /// Let's let the name field be public.
        name: String,
        _: Time,
        _: Filesystem,
        _: Metrics = build_initial_metrics(),
    }
);

decl_fixed_typemap!(
    #[fixed_typemap(dynamic)]
    pub struct ExampleMapDynamic {
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
    fn test_infallible_getters() {
        let mut map = ExampleMapNoStd::new();

        map.get_infallible_mut::<Time>().0 = 5;
        assert_eq!(map.get_infallible::<Time>().0, 5);

        assert_eq!(map.get_infallible::<Filesystem>().path, "");
        map.get_infallible_mut::<Filesystem>().path = "foo";
        assert_eq!(map.get_infallible::<Filesystem>().path, "foo");

        {
            let metrics = &map.get_infallible_mut::<Metrics>().0;
            assert_eq!(metrics.get("successes"), Some(&5));
            assert_eq!(metrics.get("failures"), Some(&10));
        }

        {
            let metrics_mut = &mut map.get_infallible_mut::<Metrics>().0;
            metrics_mut.insert("bar".into(), 15);
            assert_eq!(metrics_mut.get("bar"), Some(&15));
        }
    }

    #[test]
    fn test_fallible_getters() {
        let mut map = ExampleMapDynamic::new();

        assert!(map.get::<Time>().is_some());
        assert!(map.get_mut::<Time>().is_some());
        assert!(map.get::<Filesystem>().is_some());
        assert!(map.get_mut::<Filesystem>().is_some());
        assert!(map.get::<Metrics>().is_some());
        assert!(map.get_mut::<Metrics>().is_some());
        assert!(map.get::<u64>().is_none());
        assert!(map.get_mut::<u64>().is_none());
    }

    #[test]
    fn test_inserting_fixed() {
        let mut map = ExampleMapNoStd::new();

        // Inserting our own types should be able to replace.
        //
        // This is an entirely fixed map, which means that it always has a previous value.
        assert_eq!(map.insert(Time(5)).expect("Should insert"), Some(Time(0)));
        assert_eq!(
            map.insert::<String>("bar".into()).expect("Should insert"),
            Some("".to_string())
        );

        let mut m2 = HashMap::<String, u64>::new();
        m2.insert("inserted".into(), 1);
        assert!(map.insert(Metrics(m2)).expect("Should insert").is_some());

        // let's check that we got the values.
        assert_eq!(map.get::<Time>().unwrap().0, 5);
        assert_eq!(map.get::<String>().unwrap(), "bar");
        let metrics = map.get::<Metrics>().unwrap();
        assert_eq!(metrics.0.get("inserted").unwrap(), &1);

        // This should fail, because the typemap has no dynamicicity.
        assert!(map.insert::<u64>(0).is_err());
    }

    #[test]
    fn test_inserting_dynamic() {
        let mut map = ExampleMapDynamic::new();

        // Inserting our own types should be able to replace.
        //
        // This is an entirely fixed map, which means that it always has a previous value.
        assert_eq!(map.insert(Time(5)).expect("Should insert"), Some(Time(0)));
        assert_eq!(
            map.insert::<String>("bar".into()).expect("Should insert"),
            Some("".to_string())
        );

        let mut m2 = HashMap::<String, u64>::new();
        m2.insert("inserted".into(), 1);
        assert!(map.insert(Metrics(m2)).expect("Should insert").is_some());

        // let's check that we got the values.
        assert_eq!(map.get::<Time>().unwrap().0, 5);
        assert_eq!(map.get::<String>().unwrap(), "bar");
        let metrics = map.get::<Metrics>().unwrap();
        assert_eq!(metrics.0.get("inserted").unwrap(), &1);

        // Let's insert into the dynamic part.
        assert!(map.insert::<u64>(1).expect("Should insert").is_none());
        assert_eq!(map.get::<u64>().unwrap(), &1);
        assert_eq!(map.insert::<u64>(5).expect("Should insert").unwrap(), 1);
        assert_eq!(map.get::<u64>().unwrap(), &5);
    }
}
