use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;

use super::{Lifetime, NameComponent, TypeDef, Types};
use crate::{idents, schema};

pub type NameComponents<'sch> = Vec<NameComponent<'sch>>;

/// Computes the `TypeDef` for this schema,
/// assigning types to `types` and filling `schema.tree_handle` if necessary,
/// recursively lazy-computing `Schema` if inner types exist.
pub fn schema_to_def<'sch>(
    types: &mut Types<'sch>,
    index: &'sch schema::Index<'sch>,
    schema: &'sch schema::Schema<'sch>,
    name_comps: NameComponents<'sch>,
) -> Rc<TypeDef<'sch>> {
    let def = schema.get_type_def(&mut *types);
    if let Some(def) = def {
        return Rc::clone(def);
    }
    let def: TypeDef<'sch> = match schema.typed() {
        schema::Typed::String(s) => from_string(types, name_comps, schema, s),
        schema::Typed::Integer(s) => from_integer(s),
        schema::Typed::Number(s) => from_number(s),
        schema::Typed::Boolean(s) => from_boolean(),
        schema::Typed::Array(s) => from_array(types, name_comps, index, schema, s),
        // schema::Typed::Object(s) => from_object(handle, s),
        _ => todo!(),
    };
    let def = Rc::new(def);
    let def_id = types.insert_type(&def);
    schema.set_type_def_id(def_id);
    def
}

fn from_string<'sch>(
    types: &mut Types<'sch>,
    name_comps: NameComponents<'sch>,
    schema: &'sch schema::Schema<'sch>,
    s: &'sch schema::StringSchema<'sch>,
) -> TypeDef<'sch> {
    if let Some(enum_) = s.enum_() {
        type_enum(
            types,
            name_comps,
            schema,
            enum_.iter().map(|cow| cow.as_ref()),
        )
    } else if let Some("date-time") = s.format().as_ref().map(|cow| cow.as_ref()) {
        type_date_time()
    } else {
        type_str(s)
    }
}

fn type_enum<'sch>(
    types: &mut Types<'sch>,
    name_comps: NameComponents<'sch>,
    schema: &'sch schema::Schema<'sch>,
    enum_: impl Iterator<Item = &'sch str> + Clone + 'sch,
) -> TypeDef<'sch> {
    let handle = types.alloc_handle(name_comps.into_iter());

    let enum_ = enum_.map(|word| (word, idents::pascal(word)));
    let variants: Vec<_> = enum_
        .clone()
        .map(|(word, v_ident)| quote!(#[serde(rename = #word)] #v_ident))
        .collect();
    let arms: Vec<_> = enum_
        .clone()
        .map(|(word, v_ident)| quote!(Self::#v_ident => #word))
        .collect();

    TypeDef {
        def: handle.then_box(move |ident, _| {
            let attrs = schema_attrs(schema);
            quote! {
                #attrs
                #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
                pub enum #ident { #(#variants),* }

                impl #ident {
                    pub fn as_str(&self) -> &'static str {
                        match self { #(#arms),* }
                    }
                }
            }
        }),
        lifetime: Lifetime::empty(),
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
        lifetime: Lifetime::empty(),
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
        lifetime: Lifetime::all(),
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
        lifetime: Lifetime::empty(),
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
        lifetime: Lifetime::empty(),
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
        lifetime: Lifetime::empty(),
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
        lifetime: Lifetime::empty(),
        as_arg: Box::new(|_| quote!(bool)),
        arg_to_ser: Box::new(|_, expr| quote!(#expr)),
        as_ser: Box::new(|_| quote!(bool)),
        as_de: Box::new(|_| quote!(bool)),
        format: Box::new(|_, expr| quote!(if #expr { "true" } else { "false" })),
    }
}

fn from_array<'sch>(
    types: &mut Types<'sch>,
    name_comps: NameComponents<'sch>,
    index: &'sch schema::Index<'sch>,
    schema: &'sch schema::Schema<'sch>,
    s: &'sch schema::ArraySchema<'sch>,
) -> TypeDef<'sch> {
    let items = index
        .components()
        .resolve_schema(s.items(), |boxed| &*boxed);

    let mut name_comps_item = name_comps.clone();
    name_comps_item[0] = format!("{} Item", name_comps_item[0]).into();

    let item = schema_to_def(
        types,
        index,
        schema,
        name_comps_item,
    ); // TODO plural to singular conversion

    // Rust can't auto clone Rc for closures :(
    let item1 = Rc::clone(&item);
    let item2 = Rc::clone(&item);
    let item3 = Rc::clone(&item);

    TypeDef {
        def: Box::new(|_| quote!()),
        lifetime: Lifetime::ARG | Lifetime::SER,
        as_arg: Box::new(move |ntr| {
            let item_arg = (item1.as_arg)(ntr);
            quote!(&'ser [#item_arg])
        }),
        arg_to_ser: Box::new(|_, expr| quote!(#expr)), // TODO fix if as_arg and as_ser are different
        as_ser: Box::new(move |ntr| {
            let item_arg = (item2.as_ser)(ntr);
            quote!(&'ser [#item_arg])
        }),
        as_de: Box::new(move |ntr| {
            let item_arg = (item3.as_de)(ntr);
            quote!(Vec<#item_arg>)
        }),
        format: Box::new(|_, _| unimplemented!("could not serialize arrays")),
    }
}

/*
fn from_object(ident: &Ident, s: &'sch schema::ObjectSchema) -> TypeDef {
    todo!()
}
*/

fn schema_attrs<'sch>(schema: &'sch schema::Schema<'sch>) -> TokenStream {
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
