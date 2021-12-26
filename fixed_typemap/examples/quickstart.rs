//! The quickstart example from the docs.
use fixed_typemap::decl_fixed_typemap;

// First, define a trait to represent a plugin:
trait Plugin {
    fn run(&self);
}

// And now we do some plugin types.  We give these a `u64` value so we can demonstrate mutation.
#[derive(Default)]
struct GraphicsPlugin(u64);

#[derive(Default)]
struct SoundPlugin(u64);

#[derive(Default)]
struct NetworkingPlugin(u64);

#[derive(Default)]
struct UserProvidedPlugin(u64);

impl Plugin for GraphicsPlugin {
    fn run(&self) {
        println!("Running graphics: {}", self.0);
    }
}

impl Plugin for SoundPlugin {
    fn run(&self) {
        println!("Running sound: {}", self.0);
    }
}

impl Plugin for NetworkingPlugin {
    fn run(&self) {
        println!("Running networking: {}", self.0);
    }
}

impl Plugin for UserProvidedPlugin {
    fn run(&self) {
        println!("Running user-supplied code: {}", self.0);
    }
}

// Some plugins are always present, so we put them in the fixed part of the typemap.  But we can also have a dynamic
// section, which is where user-provided values can go.
//
// Another way to let users install their own plugins, not demonstrated here, is to define a macro that builds typemaps
// and then be generic over the kind of map provided using the InfallibleKey trait or IterableAs.
decl_fixed_typemap! {
    // We want our typemap to be dynamic, because we have an open set of user-specified values.  If we didn't specify
    // that attribute, insert would fail on new values not declared here.
    //
    // We also want to be able to iterate over our plugins to do things with them, so we ask fixed_typemap to give us a
    // helper method.  It will generate `iter_plugins` and `iter_plugins_mut` for us, as well as an implementation of
    // `IterableAs` to be used in generic code.
    #[fixed_typemap(dynamic, iterable_traits(Plugin = "iter_plugins"))]
    struct PluginMap {
        // Let's say that graphics is really important, and we want a convenient name.  It would also be possible to get
        // this without overhead via `get_infallible`, but sometimes names are convenient.
        graphics: GraphicsPlugin,
        // But we don't care about the names of the rest, because we'll only access them infrequently.
        _: SoundPlugin,
        // let's give networking a different starting value:
        _: NetworkingPlugin = NetworkingPlugin(100),
    }
}

// We can run plugins via simple iteration:
fn run_plugins(map: &PluginMap) {
    for p in map.iter_plugins() {
        p.run();
    }
}

fn main() {
    // Build our typemap:
    let mut map = PluginMap::new();

    // Now, we have everything that is in the fixed part of the map. So:
    println!("Before adding user-provided plugin");
    run_plugins(&map);

    // And we want to add one provided by our user.  Insert fails on fixed typemaps, when the type provided isn't in the
    // map, but is otherwise like std collections: either add a new value or replace.
    map.insert(UserProvidedPlugin(0))
        .expect("In this context, insert should always succeed");

    println!("After user-provided plugin");
    run_plugins(&map);

    // Now let's modify some.  Graphics is named:
    map.graphics = GraphicsPlugin(1);

    // Sound and networking are infallible at the type system level, so we can get them without going through `Option`:
    *map.get_infallible_mut::<SoundPlugin>() = SoundPlugin(2);

    // insert also updates:
    map.insert(NetworkingPlugin(10))
        .expect("Insert should always succeed in this context");

    // For the dynamic part of the map, we get back option and must go through the slower fallible getters.  We know it
    // can't fail here and this is also an example, so let's just unwrap:
    *map.get_mut::<UserProvidedPlugin>().unwrap() = UserProvidedPlugin(20);

    println!("After modification");
    run_plugins(&map);
}
