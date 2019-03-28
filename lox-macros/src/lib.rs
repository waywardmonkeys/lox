#![recursion_limit="256"]
#![feature(try_blocks)]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::DeriveInput;

use crate::derives::input::Input;


#[macro_use]
mod util;

mod derives;
mod mesh;


#[proc_macro]
pub fn mesh(input: TokenStream) -> TokenStream {
    use crate::mesh::MeshInput;

    match syn::parse::<MeshInput>(input) {
        Ok(x) => x.output().into(),
        Err(e) => e.to_compile_error().into(),
    }
}


/// Custom derive for the `Empty` trait.
///
/// See [the documentation of the `Empty` trait in `lox`][trait] for more information
/// about this derive.
///
/// [trait]: ../lox/traits/trait.Empty.html
#[proc_macro_derive(Empty)]
pub fn derive_empty(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    derives::derive_empty(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}


/// Custom derive for the `MemSink` trait.
///
/// See [the documentation of the `MemSink` trait in `lox`][trait] for more
/// information about this derive.
///
/// [trait]: ../lox/io/trait.MemSink.html
#[proc_macro_derive(MemSink, attributes(lox))]
pub fn derive_mem_sink(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    Input::from_syn(&input, "MemSink")
        .map(|i| derives::mem_sink::gen_impl(&i))
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Custom derive for the `MemSource` trait.
///
/// See [the documentation of the `MemSource` trait in `lox`][trait] for more
/// information about this derive.
///
/// [trait]: ../lox/io/trait.MemSource.html
#[proc_macro_derive(MemSource, attributes(lox))]
pub fn derive_mem_source(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    Input::from_syn(&input, "MemSource")
        .map(|i| derives::mem_source::gen_impl(&i))
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
