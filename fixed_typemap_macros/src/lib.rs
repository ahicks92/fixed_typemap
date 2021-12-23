#![allow(dead_code, unused_imports)]
use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream, Result as PResult},
    parse_quote, Token,
};

struct MapEntry {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    name: Option<syn::Ident>,
    key_type: syn::Type,
    initializer: syn::Expr,
}

struct Map {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    name: syn::Ident,
    entries: Vec<MapEntry>,
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
        let attrs = stream.call(syn::Attribute::parse_outer)?;
        let vis = stream.parse()?;
        stream.parse::<Token![struct]>()?;
        let name = stream.parse()?;

        let inner;
        syn::braced!(inner in stream);

        let entries = syn::punctuated::Punctuated::<MapEntry, Token![,]>::parse_terminated(&inner)?
            .into_iter()
            .collect();

        Ok(Map {
            attrs,
            vis,
            name,
            entries,
        })
    }
}

/// Make sure every entry in the map has a name.
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
}

/// Define the struct itself.
fn quote_struct(map: &Map) -> TokenStream2 {
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

    let attrs = &map.attrs;
    let name = &map.name;
    let vis = &map.vis;
    quote!(#(#attrs)* #vis struct #name { #(#fields),* })
}

/// Output the impls needed for the Key trait.
fn build_key_traits(map: &Map) -> TokenStream2 {
    let mut impls = vec![];

    for e in map.entries.iter() {
        let name = &map.name;
        let key_type = &e.key_type;
        impls.push(
            quote!(unsafe impl fixed_typemap_internals::Key<#name> for #key_type {
            }),
        );
    }

    quote!(#(#impls)*)
}

fn build_constructors(map: &Map) -> TokenStream2 {
    let mut joined_fields = vec![];

    for e in map.entries.iter() {
        let name = e.name.as_ref().unwrap();
        let initializer = &e.initializer;
        joined_fields.push(quote!(#name: #initializer));
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

    for (fname, const_or_mut, maybe_mut) in [
        ("get_const_ptr", quote!(const), quote!()),
        ("get_mut_ptr", quote!(mut), quote!(mut)),
    ] {
        let fident = quote::format_ident!("{}", fname);
        let clauses = type_field.iter().map(|(key, field)| {
            quote!(if core::any::TypeId::of::<K>() == core::any::TypeId::of::<#key>() {
                return &#maybe_mut self.#field
                    as *#const_or_mut #key as *#const_or_mut u8;
            })
        }).collect::<Vec<_>>();

        funcs.push(quote!(
            fn #fident<K: fixed_typemap_internals::Key<Self>>(&#maybe_mut self) -> *#const_or_mut u8 {
                #(#clauses)*
                // The `Key` trait means that this is never reachable, because we can't get a value as an input that we
                // didn't expect.
                unreachable!();
            }
        ));
    }

    quote!(#(#funcs)*)
}

fn build_safe_getters(map: &Map) -> TokenStream2 {
    let mn = &map.name;
    quote!(
        pub fn get<K: fixed_typemap_internals::Key<#mn>>(&self) -> &K {
            unsafe { &*(self.get_const_ptr::<K>() as *const K) }
        }

        pub fn get_mut<K: fixed_typemap_internals::Key<#mn>>(&mut self) -> &mut K {
            unsafe { &mut *(self.get_mut_ptr::<K>() as *mut K) }
        }
    )
}

fn build_impl_block(map: &Map) -> TokenStream2 {
    let mn = &map.name;
    let constructors = build_constructors(map);
    let unsafe_getters = build_unsafe_getters(map);
    let safe_getters = build_safe_getters(map);

    quote!(impl #mn {
        #constructors
        #unsafe_getters
        #safe_getters
    })
}

#[proc_macro]
pub fn decl_fixed_typemap(input: TokenStream) -> TokenStream {
    let mut map = syn::parse_macro_input!(input as Map);
    ensure_names(&mut map);
    let struct_def = quote_struct(&map);
    let key_traits = build_key_traits(&map);
    let impl_block = build_impl_block(&map);
    quote!(#struct_def #key_traits #impl_block).into()
}
