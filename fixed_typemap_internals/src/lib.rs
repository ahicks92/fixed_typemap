//! Internals for [fixed_typemap](https://docs.rs/fixed_typemap), a crate for producing typemaps with extended
//! functionality over a set of known types.
//!
//! This is a set of internal details used by the proc macros, and shouldn't be depended on directly.  Instead, depend
//! on and use `fixed_typemap` as this is probably what you're looking for.
#![no_std]

pub use fixed_typemap_macros::*;

/// A trait which represents the ability of a type to key a typemap infallibly.
///
/// `impl InfallibleKey<Typemap> for T` means that `T` is definitely known to be in the typemap, and as a consequence we
/// can return it without having to wrap it in `Option` and the `get_infallible` method may be used to directly retrieve
/// it.
///
/// You should never implement this trait yourself.
pub unsafe trait InfallibleKey<Map>: core::any::Any {}

/// A trait which represents the ability to iterate over a typemap with a specific trait object tuype.
///
/// In generic contexts, it is useful to be able to iterate over maps without having to know what the map contains.
/// Unfortunately, Rust coherence rules prevent us from implementing this trait as methods on the typemap, so use it
/// like:
///
///  `<dynMyTrait>::iter_as(&mymap)`.
pub trait IterableAs<'a, Map>: 'a {
    type Iter: core::iter::Iterator<Item = &'a Self>;
    type IterMut: core::iter::Iterator<Item = &'a mut Self>;

    /// Get an immutable iterator for the specified trait.
    fn iter_as(map: &'a Map) -> Self::Iter;

    /// Get a mutable iterator for the specified trait.
    fn iter_mut_as(map: &'a mut Map) -> Self::IterMut;
}
