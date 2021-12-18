#![allow(dead_code)]
use fixed_typemap::*;

struct Type1(u32);
struct Type2(u32);
struct Type3(u32);

decl_fixed_typemap!(
    struct ExampleMap {
        _: Type1,
        _: Type2 -> u32,
        _: Type3 -> u64 = 5u64,
    }
);

fn main() {}
