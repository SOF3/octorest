use proc_macro2::TokenStream;
use quote::quote;

use crate::schema;

mod from_schema;

pub fn gen(schema: schema::Index) -> (TokenStream, TokenStream) {
    // (api, types)
    (quote! {}, quote! {})
}
