#![no_std]

pub use fixed_typemap_macros::*;

/// A trait which represents the ability of a type to key a typemap infallibly.
///
/// `impl InfallibleKey<Typemap> for T` means that `T` is definitely known to be in the typemap, and as a consequence we
/// can return it without having to wrap it in `Option`. You should never implement this trait yourself.
pub unsafe trait InfallibleKey<Map>: core::any::Any {}

/// A trait which represents the ability to iterate over a typemap with a specific trait object tuype.
pub trait IterableAs<'a, Map>: 'a {
    type Iter: core::iter::Iterator<Item = &'a Self>;
    type IterMut: core::iter::Iterator<Item = &'a mut Self>;

    fn iter_as(map: &'a Map) -> Self::Iter;
    fn iter_mut_as(map: &'a mut Map) -> Self::IterMut;
}
