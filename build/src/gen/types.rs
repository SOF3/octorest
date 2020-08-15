use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use super::{schema_to_def, NameComponent, NameTree, NameTreeResolve};
use crate::schema;

#[derive(Default)]
pub struct Types<'t> {
    tree: NameTree<'t>,
    defs: Vec<TypeDef<'t>>,
}

impl<'t> Types<'t> {
    pub fn insert_schema<C: 't, I>(&mut self, index: &'t schema::Index, schema: &'t schema::Schema, name_iter: I)
    where
        NameComponent<'t>: From<C>,
        I: IntoIterator<Item = C> + Clone + 't,
    {
        let mut handle = schema.tree_handle().borrow_mut();
        if handle.is_none() {
            let def = schema_to_def(
                |last| {
                    let mut name_iter = name_iter.clone().into_iter().map(NameComponent::from);
                    let mut next = name_iter.next();
                    if last != "" {
                        next = next.map(|cow| Cow::Owned(format!("{} {}", cow, last)));
                    }

                    let h = self.tree.insert::<NameComponent<'t>, _>(next.into_iter().chain(name_iter));
                    *handle = Some(h);
                    h
                },
                self,
                index,
                schema,
            );
            self.defs.push(def);
        }
    }

    pub fn finalize(self) -> TokenStream {
        let Self { tree, defs } = self;
        let ntr = tree.resolve();

        let defs = defs.into_iter().map(|def| (def.def)(&ntr));

        quote! {
            #(#defs)*
        }
    }
}

pub struct TypeDef<'t> {
    /// The type definition, if any
    pub def: Box<dyn FnOnce(&NameTreeResolve) -> TokenStream + 't>,
    /// whether the type takes a lifetime
    pub lifetime: Lifetime,
    /// The argument type in builder, using lifetime `'ser` if `self.has_lifetime`
    pub as_arg: Box<dyn FnOnce(&NameTreeResolve) -> TokenStream + 't>,
    /// The processing code to convert `as_arg` to `as_ser`.
    /// The second argument is the expression for the value to be converted.
    pub arg_to_ser: Box<dyn FnOnce(&NameTreeResolve, TokenStream) -> TokenStream + 't>,
    /// The field type in a Serialize struct, uses lifetime `'ser` if `self.has_lifetime`
    pub as_ser: Box<dyn FnOnce(&NameTreeResolve) -> TokenStream + 't>,
    /// The field type in a Deserialize struct, uses lifetime `'de` if `self.has_lifetime`
    pub as_de: Box<dyn FnOnce(&NameTreeResolve) -> TokenStream + 't>,
    /// The preprocessing code to convert `as_ser` to a string
    /// The second argument is the expression for the value to be formatted.
    pub format: Box<dyn FnOnce(&NameTreeResolve, TokenStream) -> TokenStream + 't>,
    // add this if it's found necessary
    // /// An expression to create the default value
    // pub default: Option<Box<dyn FnOnce(&NameTreeResolve) -> TokenStream + 't>>,
}

bitflags::bitflags! {
    pub struct Lifetime: u8 {
        const ARGUMENT = 1;
        const SER = 2;
        const DESER = 4;
    }
}
