//! The last typemap you'll ever need.
//!
//! Sometimes, you want a map where the keys are types.  There are a variety of approaches to this, but they all boil
//! down to `HashMap<TypeId, Box<Any>>`. or a similar structure.  This has two problems: (1) you're always hashing your
//! types, and (2) you can't do nice things like iterate over it by traits.  This map solves both problems through the
//! [decl_fixed_typemap] proc macro.  Features:
//!
//! - Ability to specify a fixed list of types, which will be allocated inline.
//!   - these can be accessed with no overhead via `get_infallible`, which compiles down to a simple struct field borrow
//!     `&mymap.magic_field_name`.
//!   - They can also be accessed by `get`, which works additionally in dynamic contexts.
//!   - Or via the [InfallibleKey] trait, for dynamic code which wishes to accept any map that is known to contain some
//!     type.
//! - Ability to name fields of the generated struct, and to forward attributes (e.g. you can tag things with serde).
//! - If not using support for dynamic typemaps, no allocation.
//!   - In theory also `no_std` but I don't know enough about that to be sure I'm testing it right; if you want to help,
//!       finishing it will take about an hour.
//! - Ability to declare a list of traits you want to iterate by.  Mutable iteration is supported, and the returned
//!   iterators don't require boxing.
//! - As a consequence of no allocation, fixed maps don't pointer chase and are as big as the combined types.
//!
//! The big limitation is no deletion from the map.  This doesn't make sense--how do you "delete" a field in a fixed map
//! of types?  We could probably define behavior here, but it can be emulated storing `Option` in the map.
//!
//! As motivation, I wrote this to be used in an ECS which needs to allocate hundreds or thousands of typemaps for
//! component stores.  It can also be used in places where you need to fake being generic over structs which have
//! specific field names by instead using a typemap build with this crate, naming your fields, and then using newtypes
//! to enable generic functions.  The no_std story is also almost there, but I don't personally need that and so
//! finishing that up depends on interest (read: I will if you ask and are willing to help out).
//!
//! # Quickstart
//!
//! See also the [example] module for what the generated code looks like.
//!
//! Let's suppose we want to make a plugin system.  We might do it like the following, which demonstrates most of the
//! features provided by generated maps:
//!
//! ```rust
//! use fixed_typemap::decl_fixed_typemap;
//!
//! // First, define a trait to represent a plugin:
//! trait Plugin {
//!     fn run(&self);
//! }
//!
//! // And now we do some plugin types.  We give these a `u64` value so we can demonstrate mutation.
//! #[derive(Default)]
//! struct GraphicsPlugin(u64);
//!
//! #[derive(Default)]
//! struct SoundPlugin(u64);
//!
//! #[derive(Default)]
//! struct NetworkingPlugin(u64);
//!
//! #[derive(Default)]
//! struct UserProvidedPlugin(u64);
//!
//! impl Plugin for GraphicsPlugin {
//!     fn run(&self) {
//!         println!("Running graphics: {}", self.0);
//!     }
//! }
//!
//! impl Plugin for SoundPlugin {
//!     fn run(&self) {
//!         println!("Running sound: {}", self.0);
//!     }
//! }
//!
//! impl Plugin for NetworkingPlugin {
//!     fn run(&self) {
//!         println!("Running networking: {}", self.0);
//!     }
//! }
//!
//! impl Plugin for UserProvidedPlugin {
//!     fn run(&self) {
//!         println!("Running user-supplied code: {}", self.0);
//!     }
//! }
//!
//! // Some plugins are always present, so we put them in the fixed part of the typemap.  But we can also have a dynamic
//! // section, which is where user-provided values can go.
//! //
//! // Another way to let users install their own plugins, not demonstrated here, is to define a macro that builds typemaps
//! // and then be generic over the kind of map provided using the InfallibleKey trait or IterableAs.
//! decl_fixed_typemap! {
//!     // We want our typemap to be dynamic, because we have an open set of user-specified values.  If we didn't specify
//!     // that attribute, insert would fail on new values not declared here.
//!     //
//!     // We also want to be able to iterate over our plugins to do things with them, so we ask fixed_typemap to give us a
//!     // helper method.  It will generate `iter_plugins` and `iter_plugins_mut` for us, as well as an implementation of
//!     // `IterableAs` to be used in generic code.
//!     #[fixed_typemap(dynamic, iterable_traits(Plugin = "iter_plugins"))]
//!     struct PluginMap {
//!         // Let's say that graphics is really important, and we want a convenient name.  It would also be possible to get
//!         // this without overhead via `get_infallible`, but sometimes names are convenient.
//!         graphics: GraphicsPlugin,
//!         // But we don't care about the names of the rest, because we'll only access them infrequently.
//!         _: SoundPlugin,
//!         // let's give networking a different starting value:
//!         _: NetworkingPlugin = NetworkingPlugin(100),
//!     }
//! }
//!
//! // We can run plugins via simple iteration:
//! fn run_plugins(map: &PluginMap) {
//!     for p in map.iter_plugins() {
//!         p.run();
//!     }
//! }
//!
//! fn main() {
//!     // Build our typemap:
//!     let mut map = PluginMap::new();
//!
//!     // Now, we have everything that is in the fixed part of the map. So:
//!     println!("Before adding user-provided plugin");
//!     run_plugins(&map);
//!
//!     // And we want to add one provided by our user.  Insert fails on fixed typemaps, when the type provided isn't in the
//!     // map, but is otherwise like std collections: either add a new value or replace.
//!     map.insert(UserProvidedPlugin(0))
//!         .expect("In this context, insert should always succeed");
//!
//!     println!("After user-provided plugin");
//!     run_plugins(&map);
//!
//!     // Now let's modify some.  Graphics is named:
//!     map.graphics = GraphicsPlugin(1);
//!
//!     // Sound and networking are infallible at the type system level, so we can get them without going through `Option`:
//!     *map.get_infallible_mut::<SoundPlugin>() = SoundPlugin(2);
//!
//!     // insert also updates:
//!     map.insert(NetworkingPlugin(10))
//!         .expect("Insert should always succeed in this context");
//!
//!     // For the dynamic part of the map, we get back option and must go through the slower fallible getters.  We know it
//!     // can't fail here and this is also an example, so let's just unwrap:
//!     *map.get_mut::<UserProvidedPlugin>().unwrap() = UserProvidedPlugin(20);
//!
//!     println!("After modification");
//!     run_plugins(&map);
//! }
//! ```
//!
//! # So wait, how is this implemented?
//!
//! The trick here is that for infallible accesses, we can hide the borrow behind [InfallibleKey] and use the fact that
//! this is a macro to punch out a bunch of impls.  For fallible accesses, we hide the access behind some unsafe pointer
//! manipulation and a comparison with `TypeId` before falling back to a `HashMap`.  The if tree required is in theory
//! const, but Rust doesn't yet offer const `TypeId` so we can't yet make a strong guarantee.
//!
//! Trait iteration is done by storing the dynamic part of the map behind a generated cell type, which contains a boxed
//! value and a number of function pointers that look roughly like the following:
//!
//! ```ignore
//! fn convert::<ContainedT>(&Any) -> &dyn TargetTrait {
//!     // Convert to the contained type, then to a trait object.
//! }
//! ```
//!
//! Then iteration chains a fixed-sized array containing trait objects for the static part and an iterator over the map
//! and gives that back.  For fixed maps we instead chain to `Empty`, but in either case the iterator is entirely
//! allocated on the stack, the static part being `[dyn TargetTrait; field_count]` in size.
//!
//! # The Macro and What We can generate
//!
//! The macro takes a struct-like syntax.  The struct must not contain generics or lifetime parameters.  Attributes on
//! the struct are forwarded to the final struct, though care should be taken: if you're not naming all the fields
//! explicitly for example, then chances are `Serde` won't do what you want.  The syntax of a field is:
//!
//! ```ignore
//! (_ | ident): type [= expr],
//! ```
//!
//! The extensions here being `_` as a field name when you don't care about the name, and `= expression` to specify a
//! default value.  The macro requires that all fields either impl `Default` or have a provided expression.
//!
//! The `fixed_typemap` attribute can be used to control the generated struct:
//!
//! - `#[fixed_typemap(dynamic)]`: this typemap will have a dynamic section and can consequently hold any type. Requires
//!   allocation.
//! - `#[fixed_typemap(iterable_traits(path = "method_name", ... ))]`: generate a `method_name` and `method_name_mut`
//!   trait pair which will iterate over the specified trait, as well as the appropriate [IterableAs] implementations.
pub mod example;

pub use fixed_typemap_internals::{InfallibleKey, IterableAs};
pub use fixed_typemap_macros::*;
