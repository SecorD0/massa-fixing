use quote::quote;
use syn::DeriveInput;

#[macro_use]
extern crate darling;
extern crate syn;

use crate::darling::FromDeriveInput;

#[derive(FromMeta, Debug)]
struct Params {
    #[darling(multiple)]
    methods: Vec<String>
}

#[derive(Debug, FromField)]
struct MyFieldReceiver {
    /// Get the ident of the field. For fields in tuple or newtype structs or
    /// enum bodies, this can be `None`.
    ident: Option<syn::Ident>,

    /// This magic field name pulls the type from the input.
    ty: syn::Type,
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(MassaSerializationParams), forward_attrs(allow, doc, cfg), supports(struct_any))]
struct MyTraitOpts {
    ident: syn::Ident,
    data: darling::ast::Data<(), MyFieldReceiver>,
    params: Params,
}

#[proc_macro_derive(MassaSerialization, attributes(MassaSerializationParams))]
pub fn derive_massa_serialization(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    println!("into");
    let input = syn::parse_macro_input!(input as DeriveInput);
    let test = MyTraitOpts::from_derive_input(&input).unwrap();
    println!("LOL = {:#?}", test);
    println!("LOL2 = {:#?}", input);
    proc_macro::TokenStream::from(quote!())
}