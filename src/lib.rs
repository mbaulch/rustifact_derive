// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # rustifact_derive
//!
//! This crate serves to provide a derive macro for the `rustifact::ToTokenStream` trait. You should not need
//! to use this crate directly, as it's exposed via the `rustifact` crate.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed,
    Ident, Index,
};

fn get_struct_body(name: &Ident, data: &DataStruct) -> TokenStream {
    match &data.fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let mut init_toks = TokenStream::new();
            let mut fields = TokenStream::new();
            for f in named.iter() {
                let ident = &f.ident;
                init_toks.extend(quote! { let #ident = self.#ident.to_tok_stream(); });
                fields.extend(quote! { #ident: ##ident, });
            }
            quote! {
                #init_toks
                let element = rustifact::internal::quote! {
                    #name {
                        #fields
                    }
                };
                toks.extend(element);
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let mut init_toks = TokenStream::new();
            let mut fields = TokenStream::new();
            for i in 0..unnamed.len() {
                let index = Index::from(i);
                let ident = Ident::new(&format!("ident{}", i), name.span());
                init_toks.extend(quote! { let #ident = self.#index.to_tok_stream(); });
                fields.extend(quote! { ##ident, });
            }
            quote! {
                #init_toks
                let element = rustifact::internal::quote! {
                    #name ( #fields )
                };
                toks.extend(element);
            }
        }
        Fields::Unit => {
            quote! { () }
        }
    }
}

fn get_enum_body(name: &Ident, data: &DataEnum) -> TokenStream {
    let mut arms = TokenStream::new();
    for v in &data.variants {
        let ident = &v.ident;
        let toks = match &v.fields {
            Fields::Unnamed(fields_unnamed) => {
                let mut init_toks = TokenStream::new();
                let mut fields = TokenStream::new();
                let mut fields_out = TokenStream::new();
                for i in 0..fields_unnamed.unnamed.len() {
                    let id = Ident::new(&format!("ident{}", i), name.span());
                    let id_toks = Ident::new(&format!("ident{}_toks", i), name.span());
                    init_toks.extend(quote! { let #id_toks = #id.to_tok_stream(); });
                    fields.extend(quote! { #id, });
                    fields_out.extend(quote! { ##id_toks, });
                }
                if fields.is_empty() {
                    quote! {
                        #name::#ident => rustifact::internal::quote! { #name::#ident },
                    }
                } else {
                    quote! {
                        #name::#ident( #fields ) => {
                            #init_toks
                            rustifact::internal::quote! { #name::#ident( #fields_out ) }
                        },
                    }
                }
            }
            Fields::Named(_) => {
                panic!("Named fields are not yet supported");
            }
            Fields::Unit => {
                quote! { #name::#ident => rustifact::internal::quote! { #name::#ident }, }
            }
        };
        arms.extend(toks);
    }
    quote! {
        let element = match self {
            #arms
        };
        toks.extend(element);
    }
}

#[proc_macro_derive(ToTokenStream)]
pub fn derive_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let body = match &ast.data {
        Data::Struct(data) => get_struct_body(name, data),
        Data::Enum(data) => get_enum_body(name, data),
        Data::Union(_) => {
            panic!("Unions are not yet supported");
        }
    };
    let generics = &ast.generics;
    let gen_where = &generics.where_clause;
    quote! {
        impl #generics rustifact::ToTokenStream for #name #generics #gen_where {
            fn to_toks(&self, toks: &mut rustifact::internal::TokenStream) {
                #body
            }
        }
    }
    .into()
}
