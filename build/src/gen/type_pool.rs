use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use proc_macro2::{Ident, TokenStream};

use crate::{idents, schema};

#[derive(Default)]
/// Stores the types declared. This pool serves two purposes:
///
/// - Reuses the types to prevent duplicate definitions
/// - Recompute the identifier of types upon collision
///
/// # Lifetimes
/// `'t` is the lifetime of the closures to create identifiers.
pub struct TypePool<'t> {
    // TODO
    map: HashMap<schema::Schema, Rc<RefCell<SchemaEntry<'t>>>>,
    // This hashmap supports a dynamic ident mechanism.
    // A `None` entry indicates that the name was once used,
    // and should not be used for future names.
    // A `Some` entry indicates that the name is currently used.
    types: RefCell<HashMap<String, Option<Rc<RefCell<SchemaEntry<'t>>>>>>,
}

impl<'t> TypePool<'t> {
    pub fn resolve(
        &self,
        name_components: impl Iterator<Item = String> + 't,
        schema: &schema::Schema,
    ) -> Rc<SchemaEntry> {
        unimplemented!()
    }

    fn insert_entry(&self, mut entry: SchemaEntry<'t>) {
        let mut types = self.types.borrow_mut();
        while let Some(other_option) = types.get_mut(&entry.name) {
            // this name is used; we have to call entry.next_name() by the end of this iteration.

            if let Some(other) = other_option.take() {
                // this name is currently used; also next_name() the other entry.
                // take() is used such that only `None` remains for that entry.
                other.borrow_mut().next_name();
                let name = other.borrow().name.clone();
                types.insert(name, Some(other));
            }

            entry.next_name();
        }

        let _ = types.insert(entry.name.clone(), Some(Rc::new(RefCell::new(entry)))); // .expect_none()
    }

    pub fn types_ts(self) -> TokenStream {
        self.types
            .into_inner()
            .into_iter()
            .filter_map(|(_, option)| option) // only take real entries
            .map(|entry| {
                let SchemaEntry { name, ts, .. } = Rc::try_unwrap(entry)
                    .map_err(|_| "Rc should be unique at this state unique")
                    .unwrap()
                    .into_inner();
                ts(idents::pascal(&name))
            })
            .collect()
    }
}

pub struct SchemaEntry<'t> {
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

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
