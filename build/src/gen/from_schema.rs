use proc_macro2::TokenStream;
use quote::quote;

use super::{TreeHandle, TypeDef};
use crate::{idents, schema};

// note: do not read from schema.tree_handle in this function
pub fn schema_to_def<'sch>(
    handle: impl FnOnce() -> TreeHandle,
    schema: &'sch schema::Schema<'sch>,
) -> TypeDef<'sch> {
    match schema.typed() {
        schema::Typed::String(s) => from_string(handle, s),
        schema::Typed::Integer(s) => from_integer(s),
        schema::Typed::Number(s) => from_number(s),
        // schema::Typed::Boolean(s) => from_boolean(handle, s),
        // schema::Typed::Array(s) => from_array(handle, s),
        // schema::Typed::Object(s) => from_object(handle, s),
        _ => todo!(),
    }
}

fn from_string<'sch>(
    handle: impl FnOnce() -> TreeHandle,
    s: &'sch schema::StringSchema<'sch>,
) -> TypeDef<'sch> {
    if let Some(enum_) = s.enum_() {
        type_enum(enum_.iter().map(|cow| cow.as_ref()), handle)
    } else if let Some("date-time") = s.format().as_ref().map(|cow| cow.as_ref()) {
        type_date_time()
    } else {
        type_str(s)
    }
}

fn type_enum<'sch>(enum_: impl Iterator<Item = &'sch str> + Clone + 'sch, handle: impl FnOnce() -> TreeHandle) -> TypeDef<'sch> {
    let handle = handle();

    let enum_ = enum_.map(|word| (word, idents::pascal(word)));
    let variants = enum_
        .clone()
        .map(|(word, v_ident)| quote!(#[serde(rename = #word)] #v_ident));
    let arms = enum_
        .clone()
        .map(|(word, v_ident)| quote!(Self::#v_ident => #word));

    TypeDef {
        def: handle.then_box(|ident, _| {
            quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
                pub enum #ident { #(#variants),* }

                impl #ident {
                    pub fn as_str(&self) -> &'static str {
                        match self { #(#arms),* }
                    }
                }
            }
        }),
        has_lifetime: false,
        as_arg: handle.then_box(|_, path| quote!(#path)),
        arg_to_ser: Box::new(|_, expr| quote!((#expr))),
        as_ser: handle.then_box(|_, path| quote!(#path)),
        as_de: handle.then_box(|_, path| quote!(#path)),
        format: Box::new(|_, expr| quote!((#expr).as_str())),
    }
}

fn type_date_time() -> TypeDef<'static> {
    TypeDef {
        def: Box::new(|_| quote!()),
        has_lifetime: false,
        as_arg: Box::new(|_| quote!(std::time::SystemTime)),
        arg_to_ser: Box::new(|_, expr| quote!(chrono::DateTime::from(#expr))),
        as_ser: Box::new(|_| quote!(chrono::DateTime<chrono::Utc>)),
        as_de: Box::new(|_| quote!(chrono::DateTime<chrono::Utc>)),
        format: Box::new(|_, expr| quote!((#expr).to_string().as_str())),
    }
}

fn type_str<'sch>(s: &'sch schema::StringSchema<'sch>) -> TypeDef<'sch> {
    TypeDef {
        def: Box::new(|_| quote!()),
        has_lifetime: true,
        as_arg: Box::new(|_| quote!(&'ser str)),
        arg_to_ser: Box::new(|_, expr| quote!(#expr)),
        as_ser: Box::new(|_| quote!(&'ser str)),
        as_de: Box::new(|_| quote!(&'de str)),
        format: Box::new(|_, expr| quote!(#expr)),
    }
}

fn from_integer<'sch>(s: &'sch schema::IntegerSchema<'sch>) -> TypeDef<'sch> {
    if let Some("timestamp") = s.format().as_ref().map(|cow| cow.as_ref()) {
        type_timestamp()
    } else {
        type_int()
    }
}

fn type_timestamp() -> TypeDef<'static> {
    TypeDef {
        def: Box::new(|_| quote!()),
        has_lifetime: false,
        as_arg: Box::new(|_| quote!()),
        arg_to_ser: Box::new(|_, expr| quote!((#expr - std::time::UNIX_EPOCH).as_secs())),
        as_ser: Box::new(|_| quote!(u64)),
        as_de: Box::new(|_| quote!(u64)),
        format: Box::new(|_, expr| quote!((#expr).to_string().as_str())),
    }
}

fn type_int() -> TypeDef<'static> {
    TypeDef {
        def: Box::new(|_| quote!()),
        has_lifetime: false,
        as_arg: Box::new(|_| quote!(i64)),
        arg_to_ser: Box::new(|_, expr| quote!(#expr)),
        as_ser: Box::new(|_| quote!(i64)),
        as_de: Box::new(|_| quote!(i64)),
        format: Box::new(|_, expr| quote!((#expr).to_string().as_str())),
    }
}

fn from_number(s: &schema::NumberSchema) -> TypeDef<'_> {
    TypeDef {
        def: Box::new(|_| quote!()),
        has_lifetime: false,
        as_arg: Box::new(|_| quote!(f64)),
        arg_to_ser: Box::new(|_, expr| quote!(#expr)),
        as_ser: Box::new(|_| quote!(f64)),
        as_de: Box::new(|_| quote!(f64)),
        format: Box::new(|_, expr| quote!((#expr).to_string().as_str())),
    }
}

fn from_boolean() -> TypeDef<'static> {
    TypeDef {
        def: Box::new(|_| quote!()),
        has_lifetime: false,
        as_arg: Box::new(|_| quote!(bool)),
        arg_to_ser: Box::new(|_, expr| quote!(#expr)),
        as_ser: Box::new(|_| quote!(bool)),
        as_de: Box::new(|_| quote!(bool)),
        format: Box::new(|_, expr| quote!(if #expr { "true" } else { "false" })),
    }
}

/*
fn from_array(ident: &Ident, s: &schema::ArraySchema) -> TypeDef {
    todo!()
}

fn from_object(ident: &Ident, s: &schema::ObjectSchema) -> TypeDef {
    todo!()
}
*/

fn schema_attrs(schema: &schema::Schema) -> TokenStream {
    let description = schema
        .description()
        .as_ref()
        .map(|desc| quote!(#[doc = #desc]));

    let deprecated = match schema.deprecated() {
        true => quote!(#[deprecated]),
        false => quote!(),
    };

    quote! {
        #description
        #deprecated
    }
}
