#![allow(dead_code, unused_imports)]
use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use darling::FromAttributes;
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream, Parser, Result as PResult},
    parse_quote, Token,
};

#[derive(Debug, darling::FromAttributes)]
#[darling(attributes(fixed_typemap))]
struct MapAttributes {
    #[darling(default)]
    dynamic: bool,
    #[darling(default)]
    iterable_traits: std::collections::HashMap<syn::Path, syn::Ident>,
}

struct MapEntry {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    name: Option<syn::Ident>,
    key_type: syn::Type,
    initializer: syn::Expr,
}

struct Map {
    forwarded_attrs: Vec<syn::Attribute>,
    parsed_attrs: MapAttributes,
    vis: syn::Visibility,
    name: syn::Ident,
    entries: Vec<MapEntry>,
    dynamic_field_name: proc_macro2::Ident,
    dynamic_cell_name: syn::Ident,
    additional_key_constraints: Vec<syn::Path>,
}

/// Build and return a macro snippet which uses unreachable for a fast unwrap.
///
/// Very unsafe, use with care.
fn fast_unwrap(expr: TokenStream2) -> TokenStream2 {
    quote!({
        match #expr {
            Some(x) => x,
            None => unsafe { ::core::hint::unreachable_unchecked() },
        }
    })
}

impl Parse for MapEntry {
    fn parse(stream: ParseStream) -> PResult<Self> {
        let attrs = syn::Attribute::parse_outer(stream)?;
        let vis: syn::Visibility = stream.parse()?;

        let name;
        if stream.peek(Token![_]) {
            stream.parse::<Token![_]>()?;
            name = None;
        } else {
            name = Some(stream.parse()?);
        }
        stream.parse::<Token![:]>()?;
        let key_type: syn::Type = stream.parse()?;

        let mut initializer = syn::parse_quote!(Default::default());
        if stream.peek(Token![=]) {
            stream.parse::<Token![=]>()?;
            initializer = stream.parse()?;
        }
        Ok(MapEntry {
            attrs,
            vis,
            key_type,
            name,
            initializer,
        })
    }
}

impl Parse for Map {
    fn parse(stream: ParseStream) -> PResult<Self> {
        let mut forwarded_attrs = stream.call(syn::Attribute::parse_outer)?;
        let parsed_attrs = MapAttributes::from_attributes(&forwarded_attrs)
            .map_err(|e| stream.error(e.to_string()))?;

        // We must now get rid of all of the fixed_typemap attributes.
        forwarded_attrs.retain(|i| {
            for seg in i.path.segments.iter() {
                if seg.ident == "fixed_typemap" {
                    return false;
                }
            }
            true
        });

        let vis = stream.parse()?;
        stream.parse::<Token![struct]>()?;
        let name = stream.parse()?;

        let inner;
        syn::braced!(inner in stream);

        let entries = syn::punctuated::Punctuated::<MapEntry, Token![,]>::parse_terminated(&inner)?
            .into_iter()
            .collect();

        let additional_key_constraints = parsed_attrs
            .iterable_traits
            .keys()
            .map(|x| x.clone())
            .collect();

        Ok(Map {
            forwarded_attrs,
            parsed_attrs,
            vis,
            dynamic_cell_name: quote::format_ident!("{}Cell", name),
            name,
            entries,
            // This is set later, in ensure_names, but we need a dumy value for now.
            dynamic_field_name: quote::format_ident!("not_set"),
            additional_key_constraints,
        })
    }
}

/// Make sure every entry in the map has a name.
///
/// Also generate the name of the field for dynamic entries.
fn ensure_names(map: &mut Map) {
    let mut used_names = HashSet::new();
    let mut ind = 0;

    for m in map.entries.iter_mut() {
        if m.name.is_none() {
            loop {
                let n = format!("typemap_{}", ind);
                if used_names.contains(&n) {
                    ind += 1;
                    continue;
                }
                m.name = Some(syn::Ident::new(&n, proc_macro2::Span::call_site()));

                used_names.insert(n);
                break;
            }
        } else {
            used_names.insert(m.name.as_ref().unwrap().to_string());
        }
    }

    if !used_names.contains("dynamic_keys") {
        map.dynamic_field_name = quote::format_ident!("dynamic_keys");
    } else {
        let mut dyn_i: u32 = 0;
        loop {
            let candidate = format!("dynamic_keys_{}", dyn_i);
            if used_names.contains(&candidate) {
                dyn_i += 1;
                continue;
            }
            map.dynamic_field_name = quote::format_ident!("{}", candidate);
            break;
        }
    }
}

/// Builds the cell type of the map, which is used in dynamic contexts to hold map entries.
///
/// If the map isn't dynamic, returns an empty token stream.
fn build_cell_type(map: &Map) -> TokenStream2 {
    if !map.parsed_attrs.dynamic {
        return quote!();
    }

    // This type consists of a name, and then a set of function pointers which downcast to all iterable traits named as
    // the method that they go with.  The function pointers are of the form `username` and `username_mut`, and are used
    // to implement per-trait iteration.
    //
    // Each function pointer takes a `&dyn Any` and infallibly casts to the type of the object in the cell, then to the
    // trait object that type would generate. To avoid having to put named functions in a module, we just use the fact
    // that closures coerce to function pointers if they don't capture.

    let name = &map.dynamic_cell_name;

    let mut field_decls = vec![];
    let mut initializers = vec![];

    for (path, field_name) in map.parsed_attrs.iterable_traits.iter() {
        let name_mut = quote::format_ident!("{}_mut", field_name);

        field_decls.push(quote!(#field_name: fn(&dyn core::any::Any) -> &dyn #path));
        field_decls.push(quote!(#name_mut: fn(&mut dyn core::any::Any) -> &mut dyn #path));

        for (fieldname, ref_or_mut, maybe_mut) in [
            (field_name, "ref", quote!()),
            (&name_mut, "mut", quote!(mut)),
        ] {
            let dcast = quote::format_ident!("downcast_{}", ref_or_mut);
            initializers.push(quote!(
                #fieldname: |x| match x.#dcast::<K>() {
                    Some(x) => (&#maybe_mut *x) as &#maybe_mut dyn #path,
                    None => unsafe { core::hint::unreachable_unchecked() }
                }
            ));
        }
    }

    let constraints = &map.additional_key_constraints;

    quote!(struct #name {
        value: std::boxed::Box<dyn std::any::Any>,
        #(#field_decls),*
    }

    impl #name {
        fn new<K: core::any::Any + #(#constraints)+*>(value: K) -> Self {
            Self {
                value: Box::new(value),
                #(#initializers),*
            }
        }
    })
}

/// Define the struct itself.
fn build_struct(map: &Map) -> TokenStream2 {
    let mut fields = vec![];

    for e in map.entries.iter() {
        let name = e.name.as_ref().unwrap();
        let MapEntry {
            ref vis,
            ref attrs,
            ref key_type,
            ..
        } = e;
        fields.push(quote!(#(#attrs)* #vis #name : #key_type));
    }

    if map.parsed_attrs.dynamic {
        let dn = &map.dynamic_field_name;
        let cn = &map.dynamic_cell_name;
        fields.push(quote!(#dn: std::collections::HashMap<std::any::TypeId, #cn>));
    }

    let forwarded_attrs = &map.forwarded_attrs;
    let name = &map.name;
    let vis = &map.vis;
    quote!(#(#forwarded_attrs)* #vis struct #name { #(#fields),* })
}

/// Work out the type needed to iterate by a specific trait for the given map, with the given mutability.
fn build_iter_type(map: &Map, trait_name: &syn::Path, is_mut: bool) -> TokenStream2 {
    // The first step is an IntoIter for the array portion.
    let maybe_mut = if is_mut { quote!(mut) } else { quote!() };
    let arr_len = map.entries.len();
    let reftype = quote!(&#maybe_mut #trait_name);
    let static_part = quote!(core::array::IntoIter<&#maybe_mut dyn #trait_name, #arr_len>);

    let dynamic_part = if map.parsed_attrs.dynamic {
        // If the array is dynamic, we need the iterator from the hashmap, which is a map over the values to convert
        // from the cell type to the dynamic reference.
        let celltype = &map.dynamic_cell_name;
        let map_iter_type = if is_mut {
            quote!(ValuesMut)
        } else {
            quote!(Values)
        };

        quote!(core::iter::Map<std::collections::hash_map::#map_iter_type<'_, core::any::TypeId, #celltype>, for<'r> fn(&'r #maybe_mut #celltype) -> &'r #maybe_mut (dyn #trait_name + 'r)>)
    } else {
        // Otherwise, it's the empty iterator.
        quote!(core::iter::Empty<#reftype>)
    };

    // The result is the chain of these types.
    quote!(core::iter::Chain<#static_part, #dynamic_part>)
}

/// Implement all the traits we want to implement.
fn build_trait_impls(map: &Map) -> TokenStream2 {
    let name = &map.name;

    let mut impls = vec![];

    for e in map.entries.iter() {
        let key_type = &e.key_type;
        impls.push(
            quote!(unsafe impl fixed_typemap_internals::InfallibleKey<#name> for #key_type {
            }),
        );
    }

    // Implement default, for convenience.
    impls.push(quote!(
        impl core::default::Default for #name {
            fn default() -> Self { Self::new() }
        }
    ));

    quote!(#(#impls)*)
}

fn build_constructors(map: &Map) -> TokenStream2 {
    let mut joined_fields = vec![];

    for e in map.entries.iter() {
        let name = e.name.as_ref().unwrap();
        let initializer = &e.initializer;
        joined_fields.push(quote!(#name: #initializer));
    }

    if map.parsed_attrs.dynamic {
        let dn = &map.dynamic_field_name;
        joined_fields.push(quote!(#dn: Default::default()));
    }

    quote!(
        pub fn new() -> Self {
            Self {
                #(#joined_fields),*
            }
        }
    )
}

/// Build the low-level unsafe get methods.
fn build_unsafe_getters(map: &Map) -> TokenStream2 {
    let mut type_field = vec![];
    for e in map.entries.iter() {
        type_field.push((&e.key_type, e.name.as_ref().unwrap()));
    }

    let mut funcs = vec![];

    for (fname, is_mut) in [("get_const_ptr", false), ("get_mut_ptr", true)] {
        let const_or_mut = if is_mut { quote!(mut) } else { quote!(const) };

        let maybe_mut = if is_mut { quote!(mut) } else { quote!() };

        let fident = quote::format_ident!("{}", fname);
        let clauses = type_field
            .iter()
            .map(|(key, field)| {
                quote!(if core::any::TypeId::of::<K>() == core::any::TypeId::of::<#key>() {
                    return Some(&#maybe_mut self.#field
                        as *#const_or_mut #key as *#const_or_mut u8);
                })
            })
            .collect::<Vec<_>>();

        let mut final_clause = quote!(None);
        if map.parsed_attrs.dynamic {
            let suffix = if is_mut { "_mut" } else { "" };
            let map_getter = quote::format_ident!("get{}", suffix);
            let any_ref = quote::format_ident!("downcast_{}", if is_mut { "mut" } else { "ref" });
            let df = &map.dynamic_field_name;

            // Since this is a map keyed by type id, if we find the type id we expect then we found something of that
            // type.  Thus an unsafe/fast unwrap is safe.
            let downcaster = fast_unwrap(quote!((&#maybe_mut *x.value).#any_ref::<K>()));

            final_clause = quote!({
                self.#df.#map_getter(&core::any::TypeId::of::<K>()).map(|x| {
                    (#downcaster) as *#const_or_mut K as *#const_or_mut u8
                })
            });
        }

        funcs.push(quote!(
            fn #fident<K: core::any::Any>(&#maybe_mut self) -> Option<*#const_or_mut u8> {
                use core::any::Any;

                #(#clauses)*

                #final_clause
            }
        ));
    }

    quote!(#(#funcs)*)
}

fn build_infallible_getters(map: &Map) -> TokenStream2 {
    let mn = &map.name;
    let const_getter = fast_unwrap(quote!(self.get_const_ptr::<K>()));
    let mut_getter = fast_unwrap(quote!(self.get_mut_ptr::<K>()));
    let additional_constraints = &map.additional_key_constraints;
    quote!(
        pub fn get_infallible<K: fixed_typemap_internals::InfallibleKey<#mn> + #(#additional_constraints)+*>(&self) -> &K {
            unsafe { &*(#const_getter as *const K) }
        }

        pub fn get_infallible_mut<K: fixed_typemap_internals::InfallibleKey<#mn> + #(#additional_constraints)+*>(&mut self) -> &mut K {
            unsafe { &mut *(#mut_getter as *mut K) }
        }
    )
}

fn build_fallible_getters(map: &Map) -> TokenStream2 {
    let additional_constraints = &map.additional_key_constraints;
    quote!(
        pub fn get<K: core::any::Any + #(#additional_constraints)+*>(&self) -> Option<&K> {
            self.get_const_ptr::<K>()
                .map(|x| unsafe { &*(x as *const K) })
        }

        pub fn get_mut<K: core::any::Any + #(#additional_constraints)+*>(&mut self) -> Option<&mut K> {
            self.get_mut_ptr::<K>()
                .map(|x| unsafe { &mut *(x as *mut K) })
        }
    )
}

fn build_insert(map: &Map) -> TokenStream2 {
    let additional_constraints = &map.additional_key_constraints;

    let mut dynamic_clause = quote!(Err(()));
    if map.parsed_attrs.dynamic {
        let df = &map.dynamic_field_name;
        let dc = &map.dynamic_cell_name;
        let unwrapper = fast_unwrap(quote!(x.value.downcast::<K>().ok()));
        dynamic_clause = quote!(
            let tid = core::any::TypeId::of::<K>();
            Ok(self.#df.insert(tid, #dc::new(value))
                .map(|x| *(#unwrapper)))
        );
    }

    quote!(pub fn insert<K: core::any::Any + #(#additional_constraints)+*>(&mut self, mut value: K) -> Result<Option<K>, ()> {
        use core::any::Any;

        if let Some(x) = self.get_mut_ptr::<K>() {
            core::mem::swap(&mut value, unsafe { &mut *(x as *mut K) });
            return Ok(Some(value));
        }

        #dynamic_clause
    })
}

fn build_iterators(map: &Map) -> TokenStream2 {
    let mut methods = vec![];

    for (trait_path, name) in map.parsed_attrs.iterable_traits.iter() {
        for is_mut in [false, true] {
            let method_name = quote::format_ident!("{}{}", name, if is_mut { "_mut" } else { "" });
            let cell_type = &map.dynamic_cell_name;
            let maybe_mut = if is_mut { quote!(mut) } else { quote!() };
            let iter_fn = quote::format_ident!("values{}", if is_mut { "_mut" } else { "" });
            let return_type = build_iter_type(map, &trait_path, is_mut);

            // This works by having two iterators that we chain.  The first is a fixed-sized array which consists of the
            // non-dynamic fields pre-cast to the trait object.  The second consists of a map over the cell type, using
            // the inline function pointers therein to convert to the trait object as needed.
            let static_fields = map
                .entries
                .iter()
                .map(|e| {
                    let fname = &e.name.as_ref().unwrap();

                    quote!(&#maybe_mut self.#fname as &#maybe_mut dyn #trait_path)
                })
                .collect::<Vec<_>>();
            let static_fields_len = static_fields.len();

            let mut dynamic_part = quote!(let dyn_iter = core::iter::empty());
            if map.parsed_attrs.dynamic {
                let df = &map.dynamic_field_name;
                dynamic_part = quote!(
                    let dyn_ref = &#maybe_mut self.#df;
                    // We need a function which can be used as a function pointer to convert, as well as the map.  This
                    // makes it possible to name the return type.
                    fn conv<'b>(cell: &'b #maybe_mut #cell_type) -> &'b #maybe_mut dyn #trait_path {
                        (cell.#method_name)(&#maybe_mut *cell.value)
                    }

                    let dyn_iter = dyn_ref.#iter_fn().map(conv as fn(&#maybe_mut #cell_type) -> &#maybe_mut dyn #trait_path);
                )
            }

            methods.push(quote!(
                fn #method_name(&#maybe_mut self) -> #return_type {
                    let static_arr: [&#maybe_mut dyn #trait_path; #static_fields_len] = [#(#static_fields),*];
                    let static_iter = static_arr.into_iter();
                    #dynamic_part
                    static_iter.chain(dyn_iter)
                }
            ));
        }
    }

    quote!(#(#methods)*)
}

fn build_impl_block(map: &Map) -> TokenStream2 {
    let mn = &map.name;
    let constructors = build_constructors(map);
    let unsafe_getters = build_unsafe_getters(map);
    let infallible_getters = build_infallible_getters(map);
    let fallible_getters = build_fallible_getters(map);
    let insert = build_insert(map);
    let iterators = build_iterators(&map);

    quote!(impl #mn {
        #constructors
        #unsafe_getters
        #infallible_getters
        #fallible_getters
        #insert
        #iterators
    })
}

#[proc_macro]
pub fn decl_fixed_typemap(input: TokenStream) -> TokenStream {
    let mut map = syn::parse_macro_input!(input as Map);
    ensure_names(&mut map);
    let struct_def = build_struct(&map);
    let key_traits = build_trait_impls(&map);
    let cell_type = build_cell_type(&map);
    let impl_block = build_impl_block(&map);

    quote!(#struct_def
        #key_traits
        #cell_type
        #impl_block
    )
    .into()
}
