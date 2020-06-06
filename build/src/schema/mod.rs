mod index;
pub use index::*;

mod operation;
pub use operation::*;

#[allow(clippy::module_inception)]
mod schema;
pub use schema::*;

pub fn parse() -> std::io::Result<Index> {
    use std::{env, fs, path::Path};

    use serde_path_to_error as spte;

    let routes = Path::new(&env::var("CARGO_MANIFEST_DIR").expect("defined by cargo"))
        .join("routes/openapi/api.github.com");
    let path = routes.join("index.json");
    let index_file = fs::File::open(&path).map_err(err!("failed to open routes/index.json"))?;
    let mut index: Index =
        spte::deserialize(&mut serde_json::Deserializer::from_reader(index_file))
            .map_err(err!("error parsing {}", path.display()))?;

    for item in index.paths_mut().get_mut().values_mut() {
        for maybe_ref in item.get_mut().values_mut() {
            let path = match maybe_ref {
                MaybeRef::Value(_) => unreachable!("References should not have been resolved"),
                MaybeRef::Ref(r) => r.ref_path(),
            };
            let path = routes.join(path);
            let file = fs::File::open(&path).map_err(err!("failed top open {}", path.display()))?;
            let operation = spte::deserialize(&mut serde_json::Deserializer::from_reader(file))
                .map_err(err!("error parsing {}", path.display()))?;
            *maybe_ref = MaybeRef::Value(operation);
        }
    }

    Ok(index)
}
