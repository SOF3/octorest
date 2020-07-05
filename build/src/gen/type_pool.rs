use std::collections::HashMap;
use std::rc::Rc;

use proc_macro2::{Ident, TokenStream};

use crate::{idents, schema};

#[derive(Default)]
pub struct TypePool<'t> {
    map: HashMap<schema::Schema, Rc<SchemaEntry<'t>>>,
    // This hashmap supports a dynamic ident mechanism.
    // A `None` entry indicates that the name was once used,
    // and should not be used for future names.
    // A `Some` entry indicates that the name is currently used.
    types: HashMap<String, Option<SchemaEntry<'t>>>,
}

impl<'t> TypePool<'t> {
    pub fn resolve(
        &mut self,
        name_components: impl Iterator<Item = String> + 't,
        schema: &schema::Schema,
    ) -> TokenStream {
        unimplemented!()
    }

    fn insert_entry(&mut self, mut entry: SchemaEntry<'t>) {
        while let Some(other_option) = self.types.get_mut(&entry.name) {
            // this name is used; we have to call entry.next_name() by the end of this iteration.

            if let Some(mut other) = other_option.take() {
                // this name is currently used; also next_name() the other entry.
                // take() is used such that only `None` remains for that entry.
                other.next_name();
                self.types.insert(other.name.clone(), Some(other));
            }

            entry.next_name();
        }

        let _ = self.types.insert(entry.name.clone(), Some(entry)); // .expect_none()
    }

    pub fn types_ts(self) -> TokenStream {
        self.types
            .into_iter()
            .filter_map(|(_, option)| option) // only take real entries
            .map(|entry| {
                let SchemaEntry { name, ts, .. } = entry;
                ts(idents::pascal(&name))
            })
            .collect()
    }
}

struct SchemaEntry<'t> {
    name: String,
    next_names: Box<dyn Iterator<Item = String> + 't>,
    ts: Box<dyn FnOnce(Ident) -> TokenStream + 't>,
}

impl<'t> SchemaEntry<'t> {
    fn new(
        mut names: impl Iterator<Item = String> + 't,
        ts: impl FnOnce(Ident) -> TokenStream + 't,
    ) -> Self {
        let name = names.next().expect("names is empty");
        Self {
            name,
            next_names: Box::new(names),
            ts: Box::new(ts),
        }
    }

    fn next_name(&mut self) {
        let next_part = self
            .next_names
            .next()
            .expect("next_names request returns None");
        self.name = format!("{} {}", self.name, next_part);
    }
}
