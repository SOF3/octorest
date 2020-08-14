use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::{idents, schema};

pub struct TypeDef {
    pub def: TokenStream,
    pub as_arg: TokenStream,
    // takes input value as a binding calld `value`
    format_arg: TokenStream,
    // takes 'ser as lifetime
    pub as_ser: TokenStream,
    // takes 'de as lifetime
    pub as_deser: TokenStream,
}

impl TypeDef {
    pub fn format_arg(&self, arg_name: &Ident) -> TokenStream {
        let ts = &self.format_arg;
        quote! {
            {
                // rename and move #arg_name to arg
                let arg = #arg_name;
                ts
            }
        }
    }
}

pub fn type_from_schema(ident: &Ident, schema: &schema::Schema) -> TypeDef {
    let description = schema
        .description()
        .as_ref()
        .map(|desc| quote!(#[doc = #desc]));

    let deprecated = match schema.deprecated() {
        true => quote!(#[deprecated]),
        false => quote!(),
    };

    match schema.typed() {
        schema::Typed::String(s) => from_string(ident, s),
        schema::Typed::Integer(s) => from_integer(ident, s),
        schema::Typed::Number(s) => from_number(ident, s),
        schema::Typed::Boolean(s) => from_boolean(ident, s),
        schema::Typed::Array(s) => from_array(ident, s),
        schema::Typed::Object(s) => from_object(ident, s),
    }
}

fn from_string(ident: &Ident, s: &schema::StringSchema) -> TypeDef {
    if let Some(enum_) = s.enum_() {
        let (variants, arms): (Vec<_>, Vec<_>) = enum_
            .iter()
            .map(|word| {
                let ident = idents::pascal(word);
                (
                    quote!(#[serde(rename = #word)] #ident),
                    quote!(Self::#ident => #word),
                )
            })
            .unzip();
        TypeDef {
            def: quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
                pub enum #ident {
                    #(#variants),*
                }

                impl #ident {
                    pub fn as_str(&self) -> &'static str {
                        match self {
                            #(#arms),*
                        }
                    }
                }
            },
            as_arg: quote!(#ident),
            format_arg: quote!(value),
            as_ser: quote!(#ident),
            as_deser: quote!(#ident),
        }
    } else if let Some("date-time") = s.format().as_ref().map(String::as_str) {
        TypeDef {
            def: quote!(),
            as_arg: quote!(std::time::SystemTime),
            format_arg: quote!(chrono::DateTime::<chrono::Utc>::from(value).to_string()),
            as_ser: quote!(chrono::DateTime<chrono::Utc>),
            as_deser: quote!(chrono::DateTime<chrono::Utc>),
        }
    } else {
        TypeDef {
            def: quote!(),
            as_arg: quote!(&str),
            format_arg: quote!(value),
            as_ser: quote!(&'ser str),
            as_deser: quote!(&'de str),
        }
    }
}

fn from_integer(ident: &Ident, s: &schema::IntegerSchema) -> TypeDef {
    todo!()
}

fn from_number(ident: &Ident, s: &schema::NumberSchema) -> TypeDef {
    todo!()
}

fn from_boolean(ident: &Ident, s: &schema::BooleanSchema) -> TypeDef {
    todo!()
}

fn from_array(ident: &Ident, s: &schema::ArraySchema) -> TypeDef {
    todo!()
}

fn from_object(ident: &Ident, s: &schema::ObjectSchema) -> TypeDef {
    todo!()
}
