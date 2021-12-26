//! Demonstrates the generated code.
//!
//! This module serves two purposes.  First is to host the generated code on docs.rs so that the output can be
//! demonstrated.  Second is to contain all the crate's unit tests.  Viewing the source of this module demonstrates all
//! the features that exist, but see also the crate-level docs which contains a worked example.
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
    /// A typemap demonstrating the most basic functionality: it's fixed, and has no iterable traits.
    pub struct ExampleMapFixed {
        /// Let's let the name field be public.
        pub name: String,
        _: Time,
        _: Filesystem,
        _: Metrics = build_initial_metrics(),
    }
);

decl_fixed_typemap!(
    /// This typemap provides the dynamic section.
    #[fixed_typemap(dynamic)]
    pub struct ExampleMapDynamic {
        /// Let's let the name field be public.
        pub name: String,
        _: Time,
        _: Filesystem,
        _: Metrics = build_initial_metrics(),
    }
);

/// For demonstration purposes, a trait which represents things containing integral ids.
pub trait IntegralId {
    fn get_id(&self) -> u64;
    fn set_id(&mut self, id: u64);
}

#[derive(Default, derive_more::Display)]
#[display(fmt = "id1={}", _0)]
struct IdContainer1(u64);

#[derive(Default, derive_more::Display)]
#[display(fmt = "id2={}", _0)]
struct IdContainer2(u64);

#[derive(Default, derive_more::Display)]
#[display(fmt = "id3={}", _0)]
struct IdContainer3(u64);

#[derive(Default, derive_more::Display)]
#[display(fmt = "id4={}", _0)]
struct IdContainer4(u64);

macro_rules! impl_integral_id {
    ($t: ty) => {
        impl IntegralId for $t {
            fn get_id(&self) -> u64 {
                self.0
            }

            fn set_id(&mut self, id: u64) {
                self.0 = id
            }
        }
    };
    ($t: ty, $($rest: ty),+) => {
        impl_integral_id!($t);
        impl_integral_id!($($rest),*);
    }
}

impl_integral_id! {IdContainer1, IdContainer2, IdContainer3, IdContainer4}

decl_fixed_typemap! {
    /// A typemap supporting iteration by a couple different traits.
    #[fixed_typemap(dynamic, iterable_traits(
        std::fmt::Display="iter_display",
        IntegralId = "iter_integral_id",
    ))]
    pub struct IterationExampleMap {
        _: IdContainer1,
        _: IdContainer2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::IterableAs;

    #[test]
    fn test_infallible_getters() {
        let mut map = ExampleMapFixed::new();

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
        let mut map = ExampleMapFixed::new();

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

    #[test]
    fn test_iterating() {
        let mut map = IterationExampleMap::new();

        // Insert our ids.
        map.insert::<IdContainer1>(IdContainer1(1)).unwrap();
        map.insert::<IdContainer2>(IdContainer2(2)).unwrap();
        map.insert::<IdContainer3>(IdContainer3(3)).unwrap();
        map.insert::<IdContainer4>(IdContainer4(4)).unwrap();

        // Let's exercise normal iteration.
        let mut ids = map
            .iter_integral_id()
            .map(|x| x.get_id())
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, vec![1, 2, 3, 4]);

        let mut displays = map
            .iter_display()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        displays.sort();
        assert_eq!(
            displays,
            vec![
                "id1=1".to_string(),
                "id2=2".into(),
                "id3=3".into(),
                "id4=4".into()
            ]
        );

        // Now let's exercise mutable iteration, by incrementing the counters.
        for i in map.iter_integral_id_mut() {
            i.set_id(i.get_id() + 1);
        }

        let mut ids2 = map
            .iter_integral_id()
            .map(|x| x.get_id())
            .collect::<Vec<_>>();
        ids2.sort();
        assert_eq!(ids2, vec![2, 3, 4, 5]);
    }

    decl_fixed_typemap! {
        #[fixed_typemap(dynamic, iterable_traits(
            std::fmt::Display="iter_display",
            IntegralId = "iter_integral_id",
        ))]
        pub struct TestFixedIteration {
            _: IdContainer1,
            _: IdContainer2,
        }
    }

    #[test]
    fn test_iterating_fixed() {
        let mut map = TestFixedIteration::new();

        // Insert our ids.
        map.insert::<IdContainer1>(IdContainer1(1)).unwrap();
        map.insert::<IdContainer2>(IdContainer2(2)).unwrap();
        map.insert::<IdContainer3>(IdContainer3(3)).unwrap();
        map.insert::<IdContainer4>(IdContainer4(4)).unwrap();

        // Let's exercise normal iteration.
        let mut ids = map
            .iter_integral_id()
            .map(|x| x.get_id())
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, vec![1, 2, 3, 4]);

        let mut displays = map
            .iter_display()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        displays.sort();
        assert_eq!(
            displays,
            vec![
                "id1=1".to_string(),
                "id2=2".into(),
                "id3=3".into(),
                "id4=4".into()
            ]
        );

        // Now let's exercise mutable iteration, by incrementing the counters.
        for i in map.iter_integral_id_mut() {
            i.set_id(i.get_id() + 1);
        }

        let mut ids2 = map
            .iter_integral_id()
            .map(|x| x.get_id())
            .collect::<Vec<_>>();
        ids2.sort();
        assert_eq!(ids2, vec![2, 3, 4, 5]);
    }

    #[test]
    fn test_iterable_as_dynamic() {
        let mut map = IterationExampleMap::new();

        // Insert our ids.
        map.insert::<IdContainer1>(IdContainer1(1)).unwrap();
        map.insert::<IdContainer2>(IdContainer2(2)).unwrap();
        map.insert::<IdContainer3>(IdContainer3(3)).unwrap();
        map.insert::<IdContainer4>(IdContainer4(4)).unwrap();

        // Let's exercise normal iteration.
        let mut ids = <dyn IntegralId>::iter_as(&map)
            .map(|x| x.get_id())
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, vec![1, 2, 3, 4]);

        let mut displays = <dyn std::fmt::Display>::iter_as(&map)
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        displays.sort();
        assert_eq!(
            displays,
            vec![
                "id1=1".to_string(),
                "id2=2".into(),
                "id3=3".into(),
                "id4=4".into()
            ]
        );

        // Now let's exercise mutable iteration, by incrementing the counters.
        for i in <dyn IntegralId>::iter_mut_as(&mut map) {
            i.set_id(i.get_id() + 1);
        }

        let mut ids2 = <dyn IntegralId>::iter_mut_as(&mut map)
            .map(|x| x.get_id())
            .collect::<Vec<_>>();
        ids2.sort();
        assert_eq!(ids2, vec![2, 3, 4, 5]);
    }

    #[test]
    fn test_iterable_as_fixed() {
        let mut map = TestFixedIteration::new();

        // Insert our ids.
        map.insert::<IdContainer1>(IdContainer1(1)).unwrap();
        map.insert::<IdContainer2>(IdContainer2(2)).unwrap();
        map.insert::<IdContainer3>(IdContainer3(3)).unwrap();
        map.insert::<IdContainer4>(IdContainer4(4)).unwrap();

        // Let's exercise normal iteration.
        let mut ids = <dyn IntegralId>::iter_as(&map)
            .map(|x| x.get_id())
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, vec![1, 2, 3, 4]);

        let mut displays = <dyn std::fmt::Display>::iter_as(&map)
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        displays.sort();
        assert_eq!(
            displays,
            vec![
                "id1=1".to_string(),
                "id2=2".into(),
                "id3=3".into(),
                "id4=4".into()
            ]
        );

        // Now let's exercise mutable iteration, by incrementing the counters.
        for i in <dyn IntegralId>::iter_mut_as(&mut map) {
            i.set_id(i.get_id() + 1);
        }

        let mut ids2 = <dyn IntegralId>::iter_mut_as(&mut map)
            .map(|x| x.get_id())
            .collect::<Vec<_>>();
        ids2.sort();
        assert_eq!(ids2, vec![2, 3, 4, 5]);
    }
}
