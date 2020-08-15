use proc_macro2::TokenStream;
use quote::quote;
use smallvec::{smallvec, SmallVec};

use self::from_schema::schema_to_def;
pub use self::tree::TreeHandle;
use self::tree::{NameComponent, NameTree, NameTreeResolve};
use self::types::{TypeDef, Types};
use crate::schema;

mod from_schema;
mod tree;
mod types;

pub fn gen(index: schema::Index) -> TokenStream {
    let mut types = Types::default();

    crate::task("Generate types for .components.schemas", || {
        for (name, schema) in index.components().schemas() {
            types.insert_schema::<&str, SmallVec<[_; 4]>>(
                schema,
                smallvec![&**name, "schema", "comp", ""],
            );
        }
    });
    crate::task("Generate types for .components.parameters", || {
        for (name, param) in index.components().parameters() {
            types.insert_schema::<&str, SmallVec<[_; 4]>>(
                index.components().resolve_schema(param.schema()),
                smallvec![&**name, "param", "comp", ""],
            );
        }
    });
    crate::task("Generate types for .components.headers", || {
        for (name, media_type) in index.components().headers() {
            types.insert_schema::<&str, SmallVec<[_; 4]>>(
                index.components().resolve_schema(media_type.schema()),
                smallvec![&**name, "header", "comp", ""],
            );
        }
    });
    crate::task("Generate types for .components.responses", || {
        for (name, response) in index.components().responses() {
            for (mime, media_type) in response.content() {
                types.insert_schema::<&str, SmallVec<[_; 4]>>(
                    index.components().resolve_schema(media_type.schema()),
                    smallvec![&**name, "response", "comp", ""],
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
