use std::iter;
use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;

use super::{Lifetime, NameComponent, NameComponents, TypeDef, Types};
use crate::{idents, schema};

/// Computes the `TypeDef` for this schema,
/// assigning types to `types` and filling `schema.tree_handle` if necessary,
/// recursively lazy-computing `Schema` if inner types exist.
pub fn schema_to_def<'sch>(
    types: &mut Types<'sch>,
    index: &'sch schema::Index<'sch>,
    schema: &'sch schema::Schema<'sch>,
    name_comps: NameComponents<'sch>,
) -> Rc<TypeDef<'sch>> {
    log::debug!("schema_to_def(name_comps = {:?})", &name_comps);

    let def = schema.get_type_def(&mut *types);
    if let Some(def) = def {
        log::debug!("schema_to_def duplicate on {:?}", &name_comps);
        return Rc::clone(def);
    }
    let def: TypeDef<'sch> = match schema.typed() {
        schema::Typed::String(s) => from_string(types, name_comps, schema, s),
        schema::Typed::Integer(s) => from_integer(s),
        schema::Typed::Number(s) => from_number(s),
        schema::Typed::Boolean(s) => from_boolean(),
        schema::Typed::Array(s) => from_array(types, name_comps, index, schema, s),
        schema::Typed::Object(s) => from_object(types, name_comps, index, schema, s),
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
            let attrs = schema_attrs(ident.to_string().as_str(), schema);
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
        is_copy: true,
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
        is_copy: true,
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
        is_copy: true,
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
        is_copy: true,
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
        is_copy: true,
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
        is_copy: true,
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
        is_copy: true,
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
    let (items, name_comps_item) = index.components().resolve_schema(
        s.items(),
        |boxed| &*boxed,
        || {
            let mut name_comps_item = name_comps.clone();
            let first = &mut name_comps_item[0];
            first.cow = format!("{} Item", &first.cow).into();
            // TODO plural to singular conversion
            name_comps_item
        },
    );

    let item = schema_to_def(types, index, items, name_comps_item);

    let mut lifetime = Lifetime::ARG | Lifetime::SER;
    if item.lifetime.contains(Lifetime::DESER) {
        lifetime |= Lifetime::DESER;
    }

    TypeDef {
        def: Box::new(|_| quote!()),
        is_copy: false,
        lifetime,
        as_arg: Box::new({
            let item = Rc::clone(&item);
            move |ntr| {
                let item_arg = (item.as_arg)(ntr);
                quote!(&'ser [#item_arg])
            }
        }),
        arg_to_ser: Box::new(|_, expr| quote!(#expr)), // TODO fix if as_arg and as_ser are different
        as_ser: Box::new({
            let item = Rc::clone(&item);
            move |ntr| {
                let item_arg = (item.as_ser)(ntr);
                quote!(&'ser [#item_arg])
            }
        }),
        as_de: Box::new({
            let item = Rc::clone(&item);
            move |ntr| {
                let item_arg = (item.as_de)(ntr);
                quote!(Vec<#item_arg>)
            }
        }),
        format: Box::new(|_, _| unimplemented!("could not serialize arrays")),
    }
}

fn from_object<'sch>(
    types: &mut Types<'sch>,
    name_comps: NameComponents<'sch>,
    index: &'sch schema::Index<'sch>,
    schema: &'sch schema::Schema<'sch>,
    s: &'sch schema::ObjectSchema<'sch>,
) -> TypeDef<'sch> {
    let properties: Vec<(_, _, _)> = s
        .properties()
        .iter()
        .map(|(name, subschema)| {
            let (subschema, subname) =
                index.components().resolve_schema(subschema, crate::id, || {
                    iter::once(NameComponent::prepend(name.clone()))
                        .chain(name_comps.iter().cloned())
                        .collect()
                });
            let subty = schema_to_def(&mut *types, index, subschema, subname);
            (name, subschema, subty)
        })
        .collect();
    let properties = Rc::new(properties);

    let lifetime = properties
        .iter()
        .map(|(_, _, subty)| subty.lifetime)
        .fold(Lifetime::empty(), |a, b| a | b);

    // TODO process other fields

    let handle = types.alloc_handle(name_comps.into_iter());
    TypeDef {
        def: Box::new({
            let properties = Rc::clone(&properties);
            move |ntr| {
                let (ident, _) = handle.resolve(ntr);

                let ser = {
                    let ident_ser = idents::pascal(&format!("{} arg", ident));

                    let fields: TokenStream = properties
                        .iter()
                        .map(|(name, subschema, subty)| {
                            let name = idents::snake(name.as_ref());
                            let mut subty_path = (subty.as_ser)(ntr);
                            if subschema.nullable() || subschema.typed().has_default() {
                                subty_path = quote!(Option<#subty_path>);
                            }
                            quote! {
                                #name: #subty_path,
                            }
                        })
                        .collect();
                    let subty_lifetime = if lifetime.contains(Lifetime::SER) {
                        quote!(<'ser>)
                    } else {
                        quote!()
                    };

                    let field_setters: TokenStream = properties
                        .iter()
                        .map(|(name, subschema, subty)| {
                            let name = idents::snake(name.as_ref());
                            let fn_name = idents::snake(&format!("with_{}", &name));

                            let arg_ty = (subty.as_arg)(ntr);

                            let mut arg_to_ser = (subty.arg_to_ser)(ntr, quote!(value));
                            if subschema.nullable() || subschema.typed().has_default() {
                                arg_to_ser = quote!(Some(#arg_to_ser));
                            }

                            quote! {
                                pub fn #fn_name(mut self, value: #arg_ty) -> Self {
                                    self.#name = #arg_to_ser;
                                    self
                                }
                            }
                        })
                        .collect();

                    let attrs = schema_attrs(ident.to_string().as_str(), schema);
                    quote! {
                        #attrs
                        pub struct #ident_ser #subty_lifetime { #fields }

                        impl #subty_lifetime #ident_ser #subty_lifetime {
                            #field_setters
                        }
                    }
                };
                let de = {
                    let ident_de = idents::pascal(&format!("{} response", ident));

                    let fields: TokenStream = properties
                        .iter()
                        .map(|(name, subschema, subty)| {
                            let name = idents::snake(name.as_ref());
                            let mut subty_path = (subty.as_de)(ntr);
                            if subschema.nullable() || subschema.typed().has_default() {
                                subty_path = quote!(Option<#subty_path>);
                            }
                            quote! {
                                #name: #subty_path,
                            }
                        })
                        .collect();
                    let subty_lifetime = if lifetime.contains(Lifetime::DESER) {
                        quote!(<'de>)
                    } else {
                        quote!()
                    };

                    let field_getters: TokenStream = properties
                        .iter()
                        .map(|(name, subschema, subty)| {
                            let name = idents::snake(name.as_ref());

                            let ret_ty = (subty.as_de)(ntr);
                            let copy_ref = if subty.is_copy { quote!() } else { quote!(&) };

                            quote! {
                                pub fn #name(&self) -> #copy_ref #ret_ty {
                                    #copy_ref self.#name
                                }
                            }
                        })
                        .collect();

                    let attrs = schema_attrs(ident.to_string().as_str(), schema);
                    quote! {
                        #attrs
                        pub struct #ident_de #subty_lifetime { #fields }

                        impl #subty_lifetime #ident_de #subty_lifetime {
                            #field_getters
                        }
                    }
                };
                quote!(#ser #de)
            }
        }),

        is_copy: false,
        lifetime,

        // TODO only generate arg or response struct on demand
        as_arg: handle.then_box(move |ident, _| {
            // TODO fix ident vs path soundness issue
            let ser_name = idents::pascal(&format!("{} arg", ident));
            let path = quote!(crate::types::#ser_name);
            let struct_lifetime = if lifetime.contains(Lifetime::ARG) {
                quote!(<'ser>)
            } else {
                quote!()
            };
            quote!(#path #struct_lifetime)
        }),
        arg_to_ser: Box::new(|_, expr| quote!(#expr)),
        as_ser: handle.then_box(move |ident, _| {
            // TODO fix ident vs path soundness issue
            let ser_name = idents::pascal(&format!("{} arg", ident));
            let path = quote!(crate::types::#ser_name);
            let struct_lifetime = if lifetime.contains(Lifetime::SER) {
                quote!(<'ser>)
            } else {
                quote!()
            };
            quote!(#path #struct_lifetime)
        }),
        as_de: handle.then_box(move |ident, _| {
            // TODO fix ident vs path soundness issue
            let de_name = idents::pascal(&format!("{} response", ident));
            let path = quote!(crate::types::#de_name);
            let struct_lifetime = if lifetime.contains(Lifetime::DESER) {
                quote!(<'de>)
            } else {
                quote!()
            };
            quote!(#path #struct_lifetime)
        }),
        format: Box::new(|_, _| unimplemented!("cannot use object as urlencoded")),
    }
}

fn schema_attrs<'sch>(name: &'sch str, schema: &'sch schema::Schema<'sch>) -> TokenStream {
    let description = match schema.description().as_ref() {
        Some(desc) => quote!(#[doc = #desc]),
        None => {
            use heck::TitleCase;

            let title = name.to_title_case();
            quote!(#[doc = #title])
        }
    };

    let deprecated = match schema.deprecated() {
        true => quote!(#[deprecated]),
        false => quote!(),
    };

    quote! {
        #description
        #deprecated
    }
}
