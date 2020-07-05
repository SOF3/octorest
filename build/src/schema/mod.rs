#![allow(dead_code)] // TODO remove this before release to check if any info is unused

mod index;
pub use index::*;

mod operation;
pub use operation::*;

#[allow(clippy::module_inception)]
mod schema;
pub use schema::*;

pub fn parse(path: &std::path::Path) -> std::io::Result<Index> {
    use std::fs;

    use serde_path_to_error as spte;

    let index_file = fs::File::open(&path).map_err(err!("failed to open {}", path.display()))?;
    let index: Index = spte::deserialize(&mut serde_json::Deserializer::from_reader(index_file))
        .map_err(err!("error parsing {}", path.display()))?;
    Ok(index)
}
