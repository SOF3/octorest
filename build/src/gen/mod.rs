use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::{idents, schema};

mod type_pool;
use type_pool::TypePool;

struct FullOperation<'t> {
    path: &'t str,
    method: &'t str,
    operation: &'t schema::Operation,
}

pub fn gen(index: schema::Index) -> (TokenStream, TokenStream) {
    let opers = &index
        .paths()
        .get()
        .iter()
        .flat_map(|(path, item)| {
            item.get().iter().map(move |(method, operation)| FullOperation {
                path,
                method,
                operation,
            })
        })
        .collect::<Vec<_>>();

    let mods = opers
        .iter()
        .map(|fo| {
            fo.operation
                .operation_id()
                .split('/')
                .next()
                .expect("split is nonempty")
        })
        .unique();

    let mut type_pool = TypePool::default();

    let apis = mods.clone().map(|mod_| {
        let endpoints = opers
            .iter()
            .filter(|fo| fo.operation.operation_id().split('/').next() == Some(mod_))
            .map(|fo| create_endpoint(mod_, fo, &mut type_pool));

        let doc_line = &format!("{} API", heck::TitleCase::to_title_case(mod_));
        let feature_name = &format!("gh-{}", heck::KebabCase::to_kebab_case(mod_));
        let struct_ident = idents::pascal(&format!("{} API", mod_));
        quote! {
            #[doc = #doc_line]
            #[cfg(feature = #feature_name)]
            #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature_name)))]
            pub struct #struct_ident<'t> {
                main: &'t Client,
            }

            #[cfg(feature = #feature_name)]
            impl<'client> #struct_ident<'client> {
                #(#endpoints)*
            }
        }
    });

    let impls = mods.map(|mod_| {
        let method = idents::snake(mod_);
        let struct_ident = idents::pascal(&format!("{} API", mod_));
        quote! {
            pub fn #method(&self) -> #struct_ident<'_> {
                #struct_ident { main: self }
            }
        }
    });
    let impls = quote! {
        impl Client {
            #(#impls)*
        }
    };

    (
        quote! {
            use crate::Client;
            #impls
            #(#apis)*
        },
        type_pool.types_ts(),
    )
}

fn create_endpoint(tag: &str, fo: &FullOperation<'_>, type_pool: &mut TypePool) -> TokenStream {
    let method_name = idents::snake(
        &(fo.operation
            .operation_id()
            .split('/')
            .last()
            .expect("split is nonempty")
            .to_owned()
            + tag),
    );
    let summary = fo.operation.summary();
    let description = fo.operation.description();

    let http_method = idents::snake(fo.method);
    let path = format!("https://api.github.com{}", fo.path);
    let path_args = fo
        .operation
        .parameters()
        .iter()
        .filter(|param| param.location() == schema::ParameterLocation::Path)
        .map(|param| {
            let name = idents::snake(param.name());
            let uncorrected = Ident::new(param.name(), Span::call_site());
            quote!(#uncorrected = #name)
        })
        .collect::<Vec<_>>();
    let path = if path_args.is_empty() {
        quote!(#path)
    } else {
        quote!(&format!(#path, #(#path_args),*))
    };

    let accept = fo
        .operation
        .parameters()
        .iter()
        .filter(|param| param.location() == schema::ParameterLocation::Header)
        .filter(|param| param.name() == "accept")
        .map(|param| param.schema())
        .filter_map(|schema| match schema.typed() {
            schema::Typed::String(ss) => ss.default().as_ref().map(|string| string.as_str()),
            _ => None,
        })
        .next()
        .unwrap_or("application/vnd.github.v3+json");

    let input_args = fo
        .operation
        .parameters()
        .iter()
        .filter(|param| match param.location() {
            schema::ParameterLocation::Header
                if param.name() != "accept" && param.name() != "content-length" =>
            {
                true
            }
            schema::ParameterLocation::Query | schema::ParameterLocation::Path => true,
            _ => false,
        });

    let arg_list = input_args.clone().map(|param| {
        let param_name = idents::snake(param.name());
        let param_ty = type_pool.resolve(
            || idents::pascal(&format!("{} {}", fo.operation.operation_id(), param.name())),
            param.schema(),
        );
        quote!(#param_name: #param_ty)
    });

    let extra_headers = input_args
        .filter(|param| param.location() == schema::ParameterLocation::Header)
        .map(|param| {
            let header_name = param.name();
            let arg_name = idents::snake(param.name());
            quote!(builder = builder.header(#header_name, #arg_name.to_string());)
        });

    quote! {
        #[doc = #summary]
        #[doc = ""]
        #[doc = #description]
        pub async fn #method_name(&self, #(#arg_list),*) -> reqwest::Result<()> {
            let path = #path;

            let auth = self.main.get_auth_header().await?;

            let mut builder = self.main.reqwest().#http_method(path)
                .header(reqwest::header::ACCEPT, #accept);

            if let Some(header) = auth {
                builder = builder.header(reqwest::header::AUTHORIZATION, &*header);
            }

            #(#extra_headers)*

            let _result = builder
                .send()
                .await?;
            unimplemented!()
        }
    }
}
