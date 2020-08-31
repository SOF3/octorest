use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::quote;

use super::{NameTreeResolve, Types, NameComponent};
use crate::idents;
use crate::schema;

pub fn compute<'sch>(
    path: &'sch str,
    method: &'sch str,
    operation: &'sch schema::Operation<'sch>,
    types: &mut Types<'sch>,
) -> impl Fn(&NameTreeResolve) -> TokenStream + 'sch {
    let mut split = operation.operation_id().split('/');
    let tag = split.next().unwrap();
    let operation_id = split.next().expect("operationId should be in the form tag/method");
    let method_name = idents::snake(operation_id);

    let summary = operation.summary().as_ref();
    let description = operation.description().as_ref();

    let request = types.alloc_handle(vec![NameComponent::prepend(operation_id), NameComponent::prepend(tag)].into_iter());
    let def = Rc::new(TypeDef {
        def: request.then_box(|_, path| {
            quote! {
                
            }
        }),
    });
    types.insert_type(Rc::clone(def));

    move |ntr| {
        let (_, request) = request.resolve(ntr);
        quote! {
            #[doc = #summary]
            #[doc = ""]
            #[doc = #description]
            pub fn #method_name(&self) -> #request {
                #request {
                }
            }
        }
    }
}
