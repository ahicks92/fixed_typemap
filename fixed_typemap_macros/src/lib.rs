#![allow(dead_code, unused_imports)]
use proc_macro::TokenStream;

use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream, Result as PResult},
    Token,
};

struct MapEntry {
    vis: syn::Visibility,
    name: Option<syn::Ident>,
    key_type: syn::Type,
    value_type: syn::Type,
    initializer: syn::Expr,
}

impl Parse for MapEntry {
    fn parse(stream: ParseStream) -> PResult<Self> {
        let field = syn::Field::parse_named(stream)?;
        let vis = field.vis;
        let possible_name = field
            .ident
            .ok_or_else(|| stream.error("All fields must be named, or of the form _: ty"))?;
        let key_type = field.ty;
        let mut value_type = key_type.clone();

        let mut name = None;
        if possible_name == "_" && vis != syn::Visibility::Inherited {
            return Err(stream.error("Unnamed fields may not have a visibility"));
        }
        if possible_name != "_" {
            name = Some(possible_name);
        }

        // If we have a ->, the value of the field is different.
        if stream.peek(Token![->]) {
            stream.parse::<Token![->]>()?;
            value_type = stream.parse()?;
        }

        let mut initializer = syn::parse_quote!(Default::default());
        if stream.peek(Token![=]) {
            stream.parse::<Token![=]>()?;
            initializer = stream.parse()?;
        }

        Ok(MapEntry {
            vis,
            key_type,
            name,
            value_type,
            initializer,
        })
    }
}

struct Map {
    name: syn::Ident,
    entries: Vec<MapEntry>,
}

impl Parse for Map {
    fn parse(stream: ParseStream) -> PResult<Self> {
        stream.call(syn::Attribute::parse_outer)?;
        stream.parse::<Token![struct]>()?;
        let name = stream.parse()?;

        let inner;
        syn::braced!(inner in stream);

        let entries = syn::punctuated::Punctuated::<MapEntry, Token![,]>::parse_terminated(&inner)?
            .into_iter()
            .collect();

        Ok(Map { name, entries })
    }
}

#[proc_macro]
pub fn decl_fixed_typemap(input: TokenStream) -> TokenStream {
    let _ = syn::parse_macro_input!(input as Map);
    let res = quote!();
    res.into()
}
