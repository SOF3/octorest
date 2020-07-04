use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

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
            item.get()
                .iter()
                .map(move |(method, operation)| FullOperation {
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
    let mut all_types = quote!();

    let (apis, api_getters): (Vec<_>, Vec<_>) = mods
        .clone()
        .map(|mod_| {
            let (endpoints, types): (Vec<_>, Vec<_>) = opers
                .iter()
                .filter(|fo| fo.operation.operation_id().split('/').next() == Some(mod_))
                .map(|fo| create_endpoint(mod_, fo, &mut type_pool))
                .unzip();
            all_types.extend(types);

            let getter_method = idents::snake(mod_);
            let doc_line = &format!("{} API", heck::TitleCase::to_title_case(mod_));
            let feature_name = &format!("gh-{}", heck::KebabCase::to_kebab_case(mod_));
            let struct_ident = idents::pascal(&format!("{} API", mod_));
            (
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
                },
                quote! {
                    #[doc = #doc_line]
                    #[cfg(feature = #feature_name)]
                    #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature_name)))]
                    pub fn #getter_method(&self) -> #struct_ident<'_> {
                        #struct_ident { main: self }
                    }
                },
            )
        })
        .unzip();

    let impl_client = quote! {
        impl Client {
            #(#api_getters)*
        }
    };

    let types_ts = type_pool.types_ts();

    (
        quote! {
            use crate::Client;
            #impl_client
            #(#apis)*
            #all_types
        },
        quote! {
            #types_ts
        },
    )
}

fn create_endpoint(
    tag: &str,
    fo: &FullOperation<'_>,
    type_pool: &mut TypePool,
) -> (TokenStream, TokenStream) {
    let method_name_raw = fo
        .operation
        .operation_id()
        .split('/')
        .last()
        .expect("split is nonempty");

    let tag_pascal = heck::CamelCase::to_camel_case(tag);

    let method_name = idents::snake(&method_name_raw);
    let summary = fo.operation.summary();
    let description = fo.operation.description();
    let external_docs = format!(
        "- [GitHub Developer Guide]({})",
        fo.operation.external_docs().url()
    );

    let builder_name = idents::pascal(&format!("{} {} builder", tag, &method_name_raw));
    let builder_doc = format!(
        r"Request builder for `{method}`.

See the documentation of [`{method}`](struct.{tag}Api.html#method.{method})",
        method = &method_name,
        tag = &tag_pascal,
    );

    let result_type = idents::pascal(&format!("{} {} response", tag, &method_name_raw));
    let result_doc = format!(
        r"Response for `{method}`.

See the documentation of [`{method}`](struct.{tag}Api.html#method.{method}) for more information.",
        method = &method_name,
        tag = &tag_pascal,
    );

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
            quote!(#uncorrected = self.#name)
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

    let all_args = fo
        .operation
        .parameters()
        .iter()
        .map(|param| {
            let arg_name = idents::snake(param.name());
            let arg_ty = quote!(&'a str);

            (arg_name, arg_ty)
        })
        .collect::<Vec<_>>();
    // struct decl
    let arg_fields = all_args.iter().map(|(name, ty)| quote!(#name: #ty));
    // setters decl
    let arg_setters = all_args.iter().map(|(name, ty)| {
        quote! {
            pub fn #name(mut self, new: #ty) -> Self {
                self.#name = new; // TODO
                self
            }
        }
    });
    // constructor method
    let arg_constrs = all_args.iter().map(|(name, ty)| quote!(#name: ""));

    let mut variant_names = vec![];
    let mut subtys = vec![];
    let mut result_variants = vec![];
    let mut status_arms = vec![];
    let mut result_sub_types = vec![];
    for (&status, resp) in fo.operation.responses().get().iter() {
        let canon_name = http::StatusCode::from_u16(status)
            .expect("unknown HTTP status code declared")
            .canonical_reason()
            .expect("unnamed HTTP status code");
        let variant_name = idents::pascal(canon_name);
        variant_names.push(variant_name.clone());
        let subty = if status == 204 {
            quote!(())
        } else {
            idents::pascal(&format!(
                "{} {} {} response",
                tag, &method_name_raw, &variant_name
            ))
            .into_token_stream()
        };
        subtys.push(subty.clone());
        if status == 204 {
            result_variants.push(quote!(#variant_name,));
            status_arms.push(quote!(#status => #result_type::#variant_name,));
        } else {
            result_variants.push(quote!(#variant_name(#subty),));
            status_arms.push(quote! {
                #status => {
                    let body = result.bytes().await
                        .map_err(|err| crate::TransportError::Reqwest{ err })?;
                    let var = serde_json::from_slice(&body)
                        .map_err(|err| crate::TransportError::Decode{ err })?;
                    #result_type::#variant_name(var)
                },
            });

            let mut subty_doc = format!(
                r"{status} {canon} response for `{method}`.

See the documentation of [`{method}`](struct.{tag}Api.html#method.{method}) for more information.",
                method = &method_name,
                tag = &tag_pascal,
                status = status,
                canon = canon_name,
            );
            if resp.description() != "response" {
                subty_doc = format!("{}\n\n{}", resp.description(), subty_doc);
            }
            result_sub_types.push(quote! {
                #[doc = #subty_doc]
                #[derive(Debug, serde::Deserialize)]
                pub struct #subty {
                    // TODO
                }

                impl From<#subty> for #result_type {
                    fn from(variant: #subty) -> Self {
                        Self::#variant_name(variant)
                    }
                }
            });
        }
    }

    let must_use_if_multi = if result_variants.len() > 1 {
        quote!(#[must_use = "this response may be an unsuccessful variant, which should be handled"])
    } else {
        quote!()
    };

    let unwrap_if_single = if subtys.len() == 1 && subtys[0].to_string() != quote!(()).to_string() {
        let only_subty = &subtys[0];
        let only_variant = &variant_names[0];
        quote! {
            impl #result_type {
                pub fn unwrap(self) -> #only_subty {
                    match self {
                        Self::#only_variant(value) => value,
                    }
                }
            }
        }
    } else {
        quote!()
    };

    let construct_builder_method = quote! {
        #[doc = #summary]
        ///
        #[doc = #description]
        ///
        /// # See also
        #[doc = #external_docs]
        pub fn #method_name(&self) -> #builder_name {
            #builder_name {
                main: self.main,
                #(#arg_constrs,)*
                _ph: Default::default(),
            }
        }
    };

    let builder_struct = quote! {
        #[doc = #builder_doc]
        pub struct #builder_name<'t, 'a> {
            main: &'t Client,
            #(#arg_fields,)*
            _ph: std::marker::PhantomData<&'a ()>,
        }
    };

    let result_enum = quote! {
        #[doc = #result_doc]
        #[derive(Debug)]
        #must_use_if_multi
        pub enum #result_type {
            #(#result_variants)*
        }

        #unwrap_if_single

        #(#result_sub_types)*
    };

    let send_method = quote! {
        pub async fn send(self) -> Result<#result_type, crate::TransportError> {
            let path = #path;
            let auth = self.main.get_auth_header().await.map_err(|err| crate::TransportError::Reqwest{ err})?;
            let accept = #accept;
            let mut rb = self.main.reqwest()
                .#http_method(path)
                .header(reqwest::header::ACCEPT, accept);

            if let Some(header) = auth {
                rb = rb.header(reqwest::header::AUTHORIZATION, &*header);
            }

            let result = rb.send().await.map_err(|err| crate::TransportError::Reqwest{ err: err })?;
            Ok(match result.status().as_u16() {
                #(#status_arms)*
                status => {
                    return Err(crate::TransportError::UnexpectedStatus {
                        status
                    });
                },
            })
        }
    };

    (
        quote! {
            #construct_builder_method
        },
        quote! {
            #builder_struct

            impl<'t, 'a> #builder_name<'t, 'a> {
                #(#arg_setters)*

                #send_method
            }

            #result_enum
        },
    )
}
