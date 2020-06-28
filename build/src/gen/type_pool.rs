use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::{idents, schema};

#[derive(Default)]
pub struct TypePool {
    map: HashMap<schema::Schema, TokenStream>,
    types: TokenStream,
}

impl TypePool {
    pub fn resolve(
        &mut self,
        name: impl FnOnce() -> Ident,
        schema: &schema::Schema,
    ) -> TokenStream {
        if let Some(ts) = self.map.get(schema) {
            return ts.clone();
        }

        let mut ts = match schema.typed() {
            schema::Typed::String(ss) => {
                if let Some(enum_) = ss.enum_() {
                    let name = name();
                    let (variants, disp): (Vec<_>, Vec<_>) = enum_
                        .iter()
                        .map(|value| {
                            let ident = idents::pascal(value);
                            (
                                quote!(#ident),
                                quote! {
                                    Self::#ident => #value
                                },
                            )
                        })
                        .unzip();
                    self.types.extend(quote! {
                        pub enum #name {
                            #(#variants),*
                        }

                        impl ::std::fmt::Display for #name {
                            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                let lit = match self {
                                    #(#disp),*
                                };
                                f.write_str(lit)
                            }
                        }
                    });
                    quote!(crate::types::#name)
                } else {
                    quote!(&str)
                }
            }
            _ => {
                quote!(&str)
                // TODO
            }
        };
        if schema.nullable() {
            ts = quote!(Option<#ts>);
        }
        self.map.insert(schema.clone(), ts.clone());
        ts
    }

    pub fn types_ts(&self) -> TokenStream {
        self.types.clone()
    }
}
