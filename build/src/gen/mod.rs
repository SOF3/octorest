use std::rc::Rc;

use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::{idents, schema};

mod type_pool;
use type_pool::*;

struct FullOperation<'t> {
    path: &'t str,
    method: &'t str,
    operation: &'t schema::Operation,
}

impl<'t> FullOperation<'t> {
    fn params(&self) -> impl Iterator<Item = &schema::Parameter> {
        self.operation.parameters().iter()
    }
}

pub fn gen(index: schema::Index) -> (TokenStream, TokenStream) {
    // First, restructure the OpenAPI format into a list of operations
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

    // Then, group the operations by the operation tag
    // (operation ID format: "{tag}/{name}").
    let mods = opers
        .iter()
        .map(|fo| operation_id_to_tag(fo.operation.operation_id()))
        .unique()
        .collect::<Vec<_>>();

    // Type pool for reusing structs generated from Schema, to be located in `octorest::types`.
    let mut type_pool = TypePool::default();

    // Returns a closure to be called after type_pool is no longer required.
    let apis = mods.iter().map(|&mod_| {
        let getter_method = idents::snake(mod_);
        let doc_line = &format!("{} API", heck::TitleCase::to_title_case(mod_));
        let feature_name = &format!("gh-{}", heck::KebabCase::to_kebab_case(mod_));
        let tag_struct = idents::pascal(&format!("{} API", mod_));

        let mut_type_pool = &mut type_pool;
        let build_br = opers
            .iter()
            .filter(|fo| operation_id_to_tag(fo.operation.operation_id()) == mod_)
            .map(|fo| create_endpoint(mod_, &tag_struct, &feature_name, fo, mut_type_pool));

        let build_br = build_br.collect::<Vec<_>>();
        drop(mut_type_pool); // this drop statement checks that &mut type_pool is not used below

        move || {
            let (endpoints, br_types): (TokenStream, TokenStream) =
                build_br.into_iter().map(|f| f()).unzip();
            (
                // API struct for this tag
                quote! {
                    #[doc = #doc_line]
                    #[cfg(feature = #feature_name)]
                    #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature_name)))]
                    pub struct #tag_struct<'t> {
                        main: &'t Client,
                    }

                    #[cfg(feature = #feature_name)]
                    impl<'client> #tag_struct<'client> {
                        #endpoints
                    }
                },
                // Getter method for this tag from `octorest::Client`
                quote! {
                    #[doc = #doc_line]
                    #[cfg(feature = #feature_name)]
                    #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature_name)))]
                    pub fn #getter_method(&self) -> #tag_struct<'_> {
                        #tag_struct { main: self }
                    }
                },
                br_types,
            )
        }
    });

    let apis = apis.collect::<Vec<_>>(); // type_pool is resolved here

    let types_ts = type_pool.types_ts(); // type_pool is dropped in this statement

    // vec of tuples of three token streams: (api struct, Client getter, br_types)
    let apis = apis
        .into_iter()
        .map(|f| f())
        .collect::<Vec<(TokenStream, TokenStream, TokenStream)>>();

    let api_structs = apis.iter().map(|tuple| &tuple.0);
    let client_getters = apis.iter().map(|tuple| &tuple.1);
    let br_types = apis.iter().map(|tuple| &tuple.2);

    (
        quote! {
            use crate::Client;

            impl Client {
                #(#client_getters)*
            }

            #(#api_structs)*

            #(#br_types)*
        },
        types_ts,
    )
}

/// Generates code related to this endpoint.
///
///
/// # Returns
/// The return value is a closure returning a tuple.
/// Only call the tuple after `type_pool` is fully consumed.
///
/// - The first entry is the build method in the `Api` struct impl.
/// - The second entry is the builder and response definitions, outside the `impl` block.
fn create_endpoint<'t, 'p>(
    tag: &'t str,
    tag_struct: &'t Ident,
    feature_name: &'t str,
    fo: &'t FullOperation<'t>,
    type_pool: &mut TypePool<'p>, // the returned closure does not use &mut TypePool
) -> impl FnOnce() -> (TokenStream, TokenStream) + 't {
    let operation_name = operation_id_to_name(fo.operation.operation_id());

    let method_name = idents::snake(&operation_name);

    let builder_name = idents::pascal(&format!("{} {} builder", tag, &operation_name));

    let http_method = idents::snake(fo.method);

    let FormattedArgs {
        path,
        accept,
        arg_setters,
        construct_builder_method,
        builder_struct,
    } = format_args(
        &fo,
        feature_name,
        tag_struct,
        &method_name,
        &builder_name,
        type_pool,
    );

    let FormattedResp {
        response_type,
        response_subty_combs,
        response_enum,
        status_arms,
    } = format_resp(
        &fo,
        feature_name,
        tag,
        tag_struct,
        &method_name,
        &operation_name,
    );

    let send_method = quote! {
        pub async fn send(self) -> Result<#response_type, crate::TransportError> {
            let path = #path;
            let auth = self.main.get_auth_header().await.map_err(|err| crate::TransportError::Reqwest{ err })?;
            let accept = #accept;
            let mut rb = self.main.reqwest()
                .#http_method(path)
                .header(reqwest::header::ACCEPT, accept);

            if let Some(header) = auth {
                rb = rb.header(reqwest::header::AUTHORIZATION, &*header);
            }

            let result = rb.send().await.map_err(|err| crate::TransportError::Reqwest{ err: err })?;
            Ok(match result.status().as_u16() {
                #status_arms
                status => {
                    return Err(crate::TransportError::UnexpectedStatus {
                        status
                    });
                },
            })
        }
    };

    drop(type_pool);

    move || {
        let construct_builder_method = construct_builder_method();
        let builder_struct = builder_struct();
        let arg_setters = arg_setters();
        (
            quote! {
                #construct_builder_method
            },
            quote! {
                #builder_struct

                #[cfg(feature = #feature_name)]
                impl<'t, 'a> #builder_name<'t, 'a> {
                    #arg_setters

                    #send_method
                }

                #response_enum
                #response_subty_combs
            },
        )
    }
}

fn operation_id_to_tag(operation_id: &str) -> &str {
    operation_id
        .split('/')
        .next()
        .expect("Operation ID should have two parts")
}

fn operation_id_to_name(operation_id: &str) -> &str {
    operation_id
        .split('/')
        .nth(1)
        .expect("Operation ID should have two parts")
}

struct FormattedArgs<'t> {
    path: TokenStream,
    accept: TokenStream,
    arg_setters: Box<dyn FnOnce() -> TokenStream + 't>,
    construct_builder_method: Box<dyn FnOnce() -> TokenStream + 't>,
    builder_struct: Box<dyn FnOnce() -> TokenStream + 't>,
}

fn format_args<'p, 't>(
    fo: &'t FullOperation<'_>,
    feature_name: &'t str,
    tag_struct: &'t Ident,
    method_name: &'t Ident,
    builder_name: &'t Ident,
    type_pool: &mut TypePool<'p>, // the returned closures should not use type_pool
) -> FormattedArgs<'t> {
    let method_doc = format!(
        "{}\n\n{}\n\n# See also\n- [GitHub Developer Guide]({})",
        fo.operation.summary(),
        fo.operation.description(),
        fo.operation.external_docs().url()
    );
    let builder_doc = format!(
        r"Request builder for `{method}`.

See the documentation of [`{method}`](struct.{tag}Api.html#method.{method})",
        method = &method_name,
        tag = &tag_struct,
    );

    struct ProcessedArg<'t, 'p> {
        param: &'t schema::Parameter,
        field_name: Ident,
        ty: Rc<SchemaEntry<'p>>,
        required: Require,
    }

    enum Require {
        Required,
        Optional,
        /// non-arguments like Content-Length and Accept
        Accept(String),
        ContentLength,
    }

    let args = fo
        .params()
        .map(|param| {
            let field_name = idents::snake(param.name());
            let required = match param.location() {
                schema::ParameterLocation::Path => Require::Required,
                schema::ParameterLocation::Query => {
                    if param.schema().typed().has_default() {
                        Require::Optional
                    } else {
                        Require::Required
                    }
                }
                schema::ParameterLocation::Header => match param.name().as_str() {
                    "accept" => match param.schema().typed() {
                        schema::Typed::String(s) => Require::Accept(
                            s.default()
                                .as_ref()
                                .expect("Accept header must have a default value")
                                .to_owned(),
                        ),
                        _ => panic!("Unexpected Accept header type, only string is expected"),
                    },
                    "content-length" => Require::ContentLength,
                    _ => {
                        if param.schema().typed().has_default() {
                            Require::Optional
                        } else {
                            Require::Required
                        }
                    }
                },
            };
            ProcessedArg {
                param,
                required,
                field_name,
                ty: unimplemented!(), // TODO
            }
        })
        .collect::<Vec<_>>();

    FormattedArgs {
        path: {
            let path = format!("https://api.github.com{}", fo.path);
            let path_args = args
                .iter()
                .filter(|arg| arg.param.location() == schema::ParameterLocation::Path)
                .peekable();
            if path_args.peek().is_some() {
                let fmt_args = path_args
                    .map(|arg| {
                        let format_name = Ident::new(arg.param.name(), Span::call_site());
                        let field_name = idents::snake(arg.param.name());
                        quote!(#format_name = self.#field_name,)
                    })
                    .collect::<TokenStream>();
                quote!(&format!(#path, #fmt_args))
            } else {
                quote!(#path)
            }
        },
        accept: {
            let accept = args
                .iter()
                .filter_map(|arg| match &arg.required {
                    Require::Accept(accept) => Some(accept.as_str()),
                    _ => None,
                })
                .next()
                .unwrap_or("application/vnd.github.v3+json"); // TODO handle accept header properly using ."x-github".previews
            quote!(#accept)
        },
        arg_setters: Box::new(move || {
            args.iter()
                .filter(|arg| matches!(arg.required, Require::Optional))
                .map(|arg| {
                    let arg_field_name = &arg.field_name;
                    let ty = &arg.ty.name();
                    quote! {
                        pub fn #arg_field_name(mut self, new: #ty) -> Self {
                            self.#arg_field_name = new;
                            Self
                        }
                    }
                })
                .collect::<TokenStream>()
        }), // TODO

        construct_builder_method: {
            let arg_constrs = args
                .iter()
                .map(|arg| {
                    let arg_field_name = &arg.field_name;
                    match &arg.required {
                        Require::Required => quote!(#arg_field_name),
                        _ => quote!(),
                    }
                })
                .collect::<TokenStream>();
            Box::new(move || {
                let method_args = args
                    .iter()
                    .filter(|arg| matches!(arg.required, Require::Required))
                    .map(|arg| {
                        let arg_field_name = &arg.field_name;
                        let ty = &arg.ty.name();
                        quote!(#arg_field_name: #ty)
                    });
                quote! {
                    #[doc = #method_doc]
                    pub fn #method_name(&self, #(#method_args,)*) -> #builder_name {
                        #builder_name {
                            main: self.main,
                            #arg_constrs
                            _ph: Default::default(),
                        }
                    }
                }
            })
        },
        builder_struct: {
            let arg_fields = quote!(); // TODO

            Box::new(move || {
                quote! {
                    #[doc = #builder_doc]
                    #[cfg(feature = #feature_name)]
                    pub struct #builder_name<'t, 'a> {
                        main: &'t Client,
                        #arg_fields
                        _ph: std::marker::PhantomData<&'a ()>,
                    }
                }
            })
        },
    }
}

struct FormattedResp {
    response_type: Ident,
    response_subty_combs: TokenStream,
    response_enum: TokenStream,
    status_arms: TokenStream,
}

fn format_resp(
    fo: &FullOperation,
    feature_name: &str,
    tag: &str,
    tag_struct: &Ident,
    method_name: &Ident,
    operation_name: &str,
) -> FormattedResp {
    struct ProcessedResponse<'t> {
        status: u16,
        resp: &'t schema::Response,
        canon_name: &'static str,
        variant_name: Ident,
        subty: TokenStream,
    }

    let response_type = idents::pascal(&format!("{} {} response", tag, &operation_name));
    let response_doc = format!(
        r"Response for `{method}`.

See the documentation of [`{method}`](struct.{tag}Api.html#method.{method}) for more information.",
        method = &method_name,
        tag = &tag_struct,
    );

    let responses = fo
        .operation
        .responses()
        .get()
        .map(|(&status, resp)| {
            let canon_name = http::StatusCode::from_u16(status)
                .expect("unknown HTTP status code declared")
                .canonical_reason()
                .expect("unnamed HTTP status code");
            let variant_name = idents::pascal(canon_name);
            let subty = if status == 204 {
                quote!(())
            } else {
                idents::pascal(&format!(
                    "{} {} {} response",
                    tag, &operation_name, &variant_name
                ))
                .into_token_stream()
            };
            ProcessedResponse {
                status,
                resp,
                canon_name,
                variant_name,
                subty,
            }
        })
        .collect::<Vec<_>>();

    let response_variants = responses
        .iter()
        .map(|pres| {
            let variant_name = &pres.variant_name;
            let subty = &pres.subty;
            if pres.status == 204 {
                quote!(#variant_name,)
            } else {
                quote!(#variant_name(#subty),)
            }
        })
        .collect::<Vec<_>>();

    FormattedResp {
        response_type: response_type.clone(),
        response_subty_combs: {
            #[allow(clippy::range_minus_one)] // note that the empty and full subsets need not be generated
            (1..=(responses.len() - 1)).flat_map(|size| {
                responses.iter().enumerate().combinations(size).map(|mut subset| {
                    subset.sort_by_key(|(_, pres)| pres.status);
                    let subset_name = Ident::new(&format!("{}_{}", response_type,
                                                          subset.iter().map(|(_, pres)| pres.status).join("_")), Span::call_site());

                    let response_variants_subset = subset.iter().map(|(i, _)| &response_variants[*i]);

                    let reduction_methods = subset.iter().map(|(i, pres)| {
                        let ProcessedResponse{status, canon_name, variant_name, subty, ..} = &pres;
                        let status = *status;

                        let red_method_name = idents::snake(&format!("on {}", canon_name));

                        let (handler_param, handler_call, match_capture) = if status == 204 {
                            (quote!(), quote!(), quote!())
                        } else {
                            (quote!(#subty), quote!(inner), quote!((inner)))
                        };

                        if size > 1 {
                            let residue_subset_name = Ident::new(&format!("{}_{}", response_type,
                                                                          subset.iter().filter(|(j, _)| i != j)
                                                                          .map(|(_, pres)| pres.status)
                                                                          .join("_")), Span::call_site());
                            let other_variants = subset.iter()
                                .filter(|(j, _)| i != j)
                                .map(|(_, other_pres)| {
                                    let other_variant_name = &other_pres.variant_name;
                                    if other_pres.status == 204 {
                                        quote!(Self::#other_variant_name => #residue_subset_name::#other_variant_name,)
                                    } else {
                                        quote!(Self::#other_variant_name(inner) => #residue_subset_name::#other_variant_name(inner),)
                                    }
                                });

                            quote! {
                                pub fn #red_method_name(self, handler: impl FnOnce(#handler_param) -> R) -> #residue_subset_name<R> {
                                    match self {
                                        Self::Consumed(r) => #residue_subset_name::Consumed(r),
                                        Self::#variant_name #match_capture => {
                                            let ret = handler(#handler_call);
                                            #residue_subset_name::Consumed(ret)
                                        },
                                        #(#other_variants)*
                                    }
                                }
                            }
                        } else {
                            quote! {
                                pub fn #red_method_name(self, handler: impl FnOnce(#handler_param) -> R) -> R {
                                    match self {
                                        Self::Consumed(r) => r,
                                        Self::#variant_name #match_capture => handler(#handler_call)
                                    }
                                }
                            }
                        }
                    });

                    quote! {
                        #[allow(non_camel_case_types)]
                        #[must_use = "some variants have not yet been handled"]
                        #[cfg(feature = #feature_name)]
                        #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature_name)))]
                        pub enum #subset_name<R> {
                            Consumed(R),
                            #(#response_variants_subset)*
                        }

                        #[cfg(feature = #feature_name)]
                        impl<R> #subset_name<R> {
                            #(#reduction_methods)*
                        }
                    }
                }).collect::<Vec<_>>()
            }).collect()
        },
        response_enum: {
            let response_red = responses.iter().map(|pres| {
                let ProcessedResponse{status, canon_name, variant_name, subty, ..} = &pres;
                let status = *status;

                let red_method_name = idents::snake(&format!("on {}", canon_name));
                let (handler_param, handler_call, match_capture) = if status == 204 {
                    (quote!(), quote!(), quote!())
                } else {
                    (quote!(#subty), quote!(inner), quote!((inner)))
                };

                if responses.len() > 1 {
                    let residue_name = Ident::new(&format!(
                            "{}_{}", response_type, responses
                            .iter()
                            .filter(|other_pres| status != other_pres.status)
                            .map(|other_pres| other_pres.status)
                            .join("_"),
                            ), Span::call_site());
                    let other_variants = responses.iter()
                        .filter(|other_pres| status != other_pres.status)
                        .map(|other_pres| {
                            let other_variant_name = &other_pres.variant_name;
                            if other_pres.status == 204 {
                                quote!(Self::#other_variant_name => #residue_name::#other_variant_name,)
                            } else {
                                quote!(Self::#other_variant_name(inner) => #residue_name::#other_variant_name(inner),)
                            }
                        });
                    quote! {
                        pub fn #red_method_name<R>(self, handler: impl FnOnce(#handler_param) -> R) -> #residue_name<R> {
                            match self {
                                Self::#variant_name #match_capture => {
                                    let ret = handler(#handler_call);
                                    #residue_name::Consumed(ret)
                                },
                                #(#other_variants)*
                            }
                        }
                    }
                } else {
                    quote! {
                        pub fn #red_method_name<R>(self, handler: impl FnOnce(#handler_param) -> R) -> R {
                            match self {
                                Self::#variant_name #match_capture => {
                                    handler(#handler_call)
                                }
                            }
                        }
                    }
                }
            });

            let must_use_if_multi = if response_variants.len() > 1 {
                quote!(#[must_use = "this response may be an unsuccessful variant, which should be handled"])
            } else {
                quote!()
            };

            let unwrap_if_single = if responses.len() == 1 && responses[0].status != 204 {
                let ProcessedResponse {
                    variant_name: only_variant,
                    subty: only_subty,
                    ..
                } = &responses[0];
                quote! {
                    /// Unwraps this response to the only variant
                    pub fn unwrap(self) -> #only_subty {
                        match self {
                            Self::#only_variant(value) => value,
                        }
                    }
                }
            } else {
                quote!()
            };

            let response_subtys = responses
                .iter()
                .filter(|pres| pres.status != 204)
                .map(|pres| {
                    let ProcessedResponse {
                        status,
                        resp,
                        canon_name,
                        variant_name,
                        subty,
                    } = &pres;
                    let mut subty_doc = format!(
                        r"{status} {canon} response for `{method}`.

See the documentation of [`{method}`](struct.{tag}Api.html#method.{method}) for more information.",
                        method = &method_name,
                        tag = &tag_struct,
                        status = status,
                        canon = canon_name,
                    );
                    if resp.description() != "response" {
                        subty_doc = format!("{}\n\n{}", resp.description(), subty_doc);
                    }
                    quote! {
                        #[doc = #subty_doc]
                        #[derive(Debug, serde::Deserialize)]
                        #[cfg(feature = #feature_name)]
                        #[cfg_attr(feature = "internal-docsrs", doc(cfg(feature = #feature_name)))]
                        pub struct #subty {
                            // TODO
                        }

                        #[cfg(feature = #feature_name)]
                        impl From<#subty> for #response_type {
                            fn from(variant: #subty) -> Self {
                                Self::#variant_name(variant)
                            }
                        }
                    }
                });

            quote! {
                #[doc = #response_doc]
                #[derive(Debug)]
                #must_use_if_multi
                #[cfg(feature = #feature_name)]
                pub enum #response_type {
                    #(#response_variants)*
                }

                #[cfg(feature = #feature_name)]
                impl #response_type {
                    #unwrap_if_single

                    #(#response_red)*
                }

                #(#response_subtys)*
            }
        },
        status_arms: responses
            .iter()
            .map(|pres| {
                let status = pres.status;
                let variant_name = &pres.variant_name;
                if status == 204 {
                    quote!(#status => #response_type::#variant_name,)
                } else {
                    quote! {
                        #status => {
                            let body = result.bytes().await
                                .map_err(|err| crate::TransportError::Reqwest{ err })?;
                            let var = serde_json::from_slice(&body)
                                .map_err(|err| crate::TransportError::Decode{ err })?;
                            #response_type::#variant_name(var)
                        }
                    }
                }
            })
            .collect(),
    }
}
