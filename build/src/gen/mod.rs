use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use self::from_schema::schema_to_def;
use self::tree::{NameComponent, NameTree, NameTreeResolve, TreeHandle};
use self::types::Lifetime;
pub use self::types::{TypeDef, Types};
use crate::schema;

mod from_schema;
mod tree;
mod types;

macro_rules! cow_iter {
    ($size:literal : $($args:expr),* $(,)?) => {{
        struct Iter<'t>([&'t str; $size], usize);

        impl<'t> Iterator for Iter<'t> {
            type Item = Cow<'t, str>;

            fn next(&mut self) -> Option<Cow<'t, str>> {
                let option = self.0.get(self.1)
                    .map(|&cow| Cow::from(cow));
                self.1 += 1;
                option
            }
        }

        Iter([$($args),*], 0)
    }};
}

pub fn gen<'sch>(index: &'sch schema::Index<'sch>) -> TokenStream {
    let mut types = Types::default();

    crate::task("Generate types for .components.schemas", || {
        for (name, schema) in index.components().schemas() {
            schema_to_def(
                &mut types,
                index,
                schema,
                cow_iter![3: &**name, "schema", "comp"],
            );
        }
    });
    crate::task("Generate types for .components.parameters", || {
        for (name, param) in index.components().parameters() {
            schema_to_def(
                &mut types,
                index,
                index.components().resolve_schema(param.schema(), crate::id),
                cow_iter![3: &**name, "param", "comp"],
            );
        }
    });
    crate::task("Generate types for .components.headers", || {
        for (name, media_type) in index.components().headers() {
            schema_to_def(
                &mut types,
                index,
                index
                    .components()
                    .resolve_schema(media_type.schema(), crate::id),
                cow_iter![3: &**name, "header", "comp"],
            );
        }
    });
    crate::task("Generate types for .components.responses", || {
        for (name, response) in index.components().responses() {
            for (mime, media_type) in response.content() {
                schema_to_def(
                    &mut types,
                    index,
                    index
                        .components()
                        .resolve_schema(media_type.schema(), crate::id),
                    cow_iter![3: &**name, "response", "comp"],
                );
            }
        }
    });

    let types = types.finalize();

    let ret = quote! {
        pub mod types {
            #types
        }
    };

    dbg!(ret.to_string());
    ret
}

fn iter_change_first<'t, T: 't>(
    mut iter: impl Iterator<Item = T> + 't,
    change: impl FnOnce(T) -> T,
) -> impl Iterator<Item = T> + 't {
    // boxing needed because rustc can't resolve recursive generics
    Box::new(iter.next()
        .map(change)
        .into_iter()
        .chain(iter)) as Box<dyn Iterator<Item = T> + 't>
}
