#![no_std]

pub use fixed_typemap_macros::*;

/// A trait which represents the ability of a type to key a typemap.
///
/// `Key<TypeMap>` means that the type this trait is implemented for is in the
/// specified typemap, and produces the specific kind of value.  The macros
/// implement this trait for you; see the documentation there for details.
///
/// You should never implement this trait yourself.
pub unsafe trait Key<Map>: Sized + core::any::Any {
}
