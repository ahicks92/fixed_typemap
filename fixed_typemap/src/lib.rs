#![no_std]

/// A trait which represents the ability of a type to key a typemap.
///
/// `Key<TypeMap>` means that the type this trait is implemented for is in the
/// specified typemap, and produces the specific kind of value.  The macros
/// implement this trait for you; see the documentation there for details.
///
/// You should never implement this trait yourself.
pub unsafe trait Key<Map>: Sized + core::any::Any {
    type Value: Sized;
}

/// Trait to support mutable borrow splitting.
///
/// It is possible to split borrows for a given typemap using tuples of types.
/// This trait is used to provide verification at the type system level that the
/// specified keys are all keys of the typemap.  See crate documentation for
/// details.
pub unsafe trait SplittableBorrow<Map> {}
