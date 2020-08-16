use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;

use super::{NameComponent, NameTree, NameTreeResolve, TreeHandle};

#[derive(Default)]
pub struct Types<'t> {
    tree: NameTree<'t>,
    defs: Vec<Rc<TypeDef<'t>>>,
}

impl<'t> Types<'t> {
    pub fn alloc_handle(
        &mut self,
        name_comps: impl Iterator<Item = NameComponent<'t>> + 't,
    ) -> TreeHandle {
        self.tree.insert(name_comps)
    }

    pub fn insert_type(&mut self, type_def: &Rc<TypeDef<'t>>) {
        self.defs.push(Rc::clone(type_def));
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
    pub as_arg: Box<dyn Fn(&NameTreeResolve) -> TokenStream + 't>,
    /// The processing code to convert `as_arg` to `as_ser`.
    /// The second argument is the expression for the value to be converted.
    pub arg_to_ser: Box<dyn Fn(&NameTreeResolve, TokenStream) -> TokenStream + 't>,
    /// The field type in a Serialize struct, uses lifetime `'ser` if `self.has_lifetime`
    pub as_ser: Box<dyn Fn(&NameTreeResolve) -> TokenStream + 't>,
    /// The field type in a Deserialize struct, uses lifetime `'de` if `self.has_lifetime`
    pub as_de: Box<dyn Fn(&NameTreeResolve) -> TokenStream + 't>,
    /// The preprocessing code to convert `as_ser` to a string
    /// The second argument is the expression for the value to be formatted.
    pub format: Box<dyn Fn(&NameTreeResolve, TokenStream) -> TokenStream + 't>,
    // add this if it's found necessary
    // /// An expression to create the default value
    // pub default: Option<Box<dyn Fn(&NameTreeResolve) -> TokenStream + 't>>,
}

bitflags::bitflags! {
    pub struct Lifetime: u8 {
        const ARG = 1;
        const SER = 2;
        const DESER = 4;
    }
}
