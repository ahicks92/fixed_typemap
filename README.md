# fixed_typemap

![GitHub Actions](https://github.com/ahicks92/fixed_typemap/actions/workflows/ci.yaml/badge.svg)
[docs.rs](https://docs.rs/fixed_typemap) [GitHub Sponsors](https://github.com/sponsors/ahicks92)

Implements typemaps that support a lot of extra funcctionality using procedural macros.  docs.rs has a lot more than
this readme, including a mini-tutorial and worked example.  You can use this to:

- Implement fixed typemaps which don't allocate, and store all their members inline initialized to default values.
- Use this to implement something like the fields-in-traits proposal, where you "name" fields generically using types.
- Generate iteration helpers which can iterate over the typemap as trait objects for any number of traits (e.g. this can
  replace `HashMap<TypeId, Box<dyn MyTrait>>`, bringing the rest of the functionality along for the ride and also let
  you do as many traits as you want at once).
- Add an optional dynamic section which uses a `HashMap` to enable storing any type.

`no_std` support is WIP in the sense that it's basically done but I don't know enough to reliably finish it.  Doing so
shouldn't be hard and I'm very open to contributions which do so and preferrably provide CI configuration.
