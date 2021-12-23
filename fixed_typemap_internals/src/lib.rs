#![no_std]

pub use fixed_typemap_macros::*;

/// A trait which represents the ability of a type to key a typemap infallibly.
///
/// `impl InfallibleKey<Typemap> for T` means that `T` is definitely known to be in the typemap, and as a consequence we
/// can return it without having to wrap it in `Option`. You should never implement this trait yourself.
pub unsafe trait InfallibleKey<Map>: Sized + core::any::Any {}
