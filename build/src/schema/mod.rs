#![allow(dead_code)] // TODO remove this before release to check if any info is unused

mod index;
pub use index::*;

// #[path = "operation_expand.rs"]
mod operation;
pub use operation::*;

#[allow(clippy::module_inception)]
mod schema;
pub use schema::*;

mod maybe_ref;
use maybe_ref::{MaybeRef, Ref};

pub fn parse<'sch>(input: &'sch str) -> std::io::Result<Index<'sch>> {
    use serde_path_to_error as spte;

    let index: Index = spte::deserialize(&mut serde_json::Deserializer::from_str(input))
        .map_err(err!("error parsing api.github.com.json"))?;
    Ok(index)
}
