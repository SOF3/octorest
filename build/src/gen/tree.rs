use std::borrow::{Cow};
use std::collections::BTreeMap;
use std::iter;

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::idents;

/// A data structure that tracks string names and increases their size when duplicate
///
/// # Lifetimes
/// `'t` is the lifetime of name iterators
#[derive(Default)]
pub struct NameTree<'t> {
    map: BTreeMap<String, Option<TreeEntry<'t>>>,
    next_handle: usize,
}

impl<'t> NameTree<'t> {
    /// `name_iter` is the iterator that yields next name copmonents.
    /// In the output ident, the order of strings created by `name_iter` will be reversed.  `name_iter` must not yield empty strings.
    pub fn insert<C: 't, I>(&mut self, name_iter: I) -> TreeHandle
    where
        NameComponent<'t>: From<C>,
        I: IntoIterator<Item = C> + 't,
    {
        let mut name_iter = name_iter
            .into_iter()
            .map(NameComponent::from)
            .chain(iter::once(NameComponent::Borrowed("")));

        let mut key = idents::pascal(&name_iter.next().expect("name_iter is empty")).to_string();

        while let Some(option) = self.map.get_mut(&key) {
            if let Some(mut other_tree_entry) = option.take() {
                // tree_entry is currently called `key` as well.
                // Pushing to `key + [tree_entry.name_iter.next()]` is sufficient,
                // because existence of `key` implies all `key + *` do not exist.

                let mut other_key = key.clone();
                log::debug!("Key collision: {}", &other_key);

                other_key = idents::pascal(&other_tree_entry
                        .name_iter
                        .next()
                        .expect("name_iter prefix detected"))
                        .to_string() + &other_key; // prepend
                self.map.insert(other_key.into(), Some(other_tree_entry));
            }
            // else, tree_entry was already pushed and we do not need to push it again.

            key = idents::pascal(&name_iter.next().expect("name_iter prefix detected")).to_string() + &key; // prepend
        }

        let handle = TreeHandle(self.next_handle);
        self.next_handle += 1;

        let insert = self.map.insert(
            key.into(),
            Some(TreeEntry {
                name_iter: Box::new(name_iter),
                handle,
            }),
        );
        if insert.is_some() {
            panic!("self.map.get_mut was None but insert is some");
        }

        handle
    }

    pub fn resolve(self) -> NameTreeResolve {
        let mut ret = (0..self.next_handle).map(|_| None).collect::<Vec<_>>();
        for (key, value) in self.map {
            if let Some(tree_entry) = value {
                let ident = proc_macro2::Ident::new(&key, proc_macro2::Span::call_site());
                let path = quote!(crate::types::#ident);
                ret[tree_entry.handle.0] = Some(ResolvedEntry { ident, path });
            }
        }
        NameTreeResolve(ret)
    }
}

pub type NameComponent<'t> = Cow<'t, str>;

pub type NameComponents<'t> = Vec<NameComponent<'t>>;

struct TreeEntry<'t> {
    name_iter: Box<dyn Iterator<Item = NameComponent<'t>> + 't>,
    handle: TreeHandle,
}

#[derive(Debug)]
pub struct NameTreeResolve(Vec<Option<ResolvedEntry>>);

#[derive(Debug)]
struct ResolvedEntry {
    ident: Ident,
    path: TokenStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeHandle(usize);

impl TreeHandle {
    pub fn then<R, F: FnOnce(&Ident, &TokenStream) -> R>(self, f: F) -> TreeHandleThen<R, F> {
        TreeHandleThen(self.0, f)
    }

    pub fn resolve<'t>(self, ntr: &'t NameTreeResolve) -> (&'t Ident, &'t TokenStream) {
        let entry = ntr.0[self.0].as_ref().expect("Unresolved ident");
        (&entry.ident, &entry.path)
    }

    pub fn then_box<'t, R: 't, F: Fn(&Ident, &TokenStream) -> R + 't>(
        self,
        f: F,
    ) -> Box<dyn Fn(&NameTreeResolve) -> R + 't> {
        Box::new(move |ntr| self.then(&f).resolve(ntr))
    }

    pub fn then_format<'t>(
        self,
        f: impl Fn(&Ident, &TokenStream, TokenStream) -> TokenStream + 't,
    ) -> Box<dyn Fn(&NameTreeResolve, TokenStream) -> TokenStream + 't> {
        Box::new(move |ntr, expr| {
            let f = &f;
            self.then(move |ident, path| f(ident, path, expr))
                .resolve(ntr)
        })
    }
}

pub struct TreeHandleThen<R, F: FnOnce(&Ident, &TokenStream) -> R>(usize, F);

impl<R, F: FnOnce(&Ident, &TokenStream) -> R> TreeHandleThen<R, F> {
    pub fn resolve(self, ntr: &NameTreeResolve) -> R {
        let Self(handle, f) = self;
        let ResolvedEntry { ident, path } = ntr.0[handle].as_ref().expect("Unresolved ident");
        f(ident, path)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_tree() {
        let mut tree = super::NameTree::default();
        let handle1 = tree.insert(["xa", "xb", "xc", "xf"].into_iter().copied());
        let handle2 = tree.insert(["xa", "xb", "xd", "xe"].into_iter().copied());

        let abc = handle1.then(|ident, _| ident.to_string());
        let abd = handle2.then(|ident, _| ident.to_string());

        let ntr = tree.resolve();

        assert_eq!(abc.resolve(&ntr).as_str(), "XaXbXc");
        assert_eq!(abd.resolve(&ntr).as_str(), "XaXbXd");
    }
}
