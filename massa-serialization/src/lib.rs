use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[macro_use]
extern crate darling;
extern crate syn;

use crate::darling::FromDeriveInput;

#[derive(FromMeta, Debug)]
struct Methods {
    serialize: syn::Ident,
    deserialize: syn::Ident
}

#[derive(FromMeta, Debug)]
struct Params {
    #[darling(multiple)]
    methods: Vec<Methods>,
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
#[darling(
    attributes(MassaSerializationParams),
    forward_attrs(allow, doc, cfg),
    supports(struct_any)
)]
struct MyTraitOpts {
    ident: syn::Ident,
    data: darling::ast::Data<(), MyFieldReceiver>,
    params: Params,
}

#[proc_macro_derive(MassaSerialization, attributes(MassaSerializationParams))]
pub fn derive_massa_serialization(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    println!("into");
    let input = syn::parse_macro_input!(input as DeriveInput);
    let structure = MyTraitOpts::from_derive_input(&input).unwrap();
    println!("test = {:#?}", structure);
    let struct_type = structure.ident;

    let code = if structure.params.methods.is_empty() {
        quote!(
            impl #struct_type {
                fn to_bytes_compact(&self) {
                    println!("in lol");
                }

                fn from_bytes_compact(&self) {
                    println!("in lol");
                }
            }
        )
    } else {
        let mut code_temp: TokenStream = TokenStream::new();
        for method in structure.params.methods {
            let method_serialize = method.serialize;
            let method_deserialize = method.deserialize;
            code_temp.extend(quote!(
                impl #struct_type {
                    fn #method_serialize(&self) {
                        println!("in lol");
                    }

                    fn #method_deserialize(&self) {
                        println!("in lol");
                    }
                }
            ))
        }
        code_temp
    };
    proc_macro::TokenStream::from(code)
}
