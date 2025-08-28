use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, FnArg, ItemFn, Pat, PatType, parse_macro_input};

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let mut transformers: Vec<String> = Vec::new();
    let mut arg_bindings: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut new_fn_args: Vec<FnArg> = Vec::new();

    for arg in &sig.inputs {
        if let FnArg::Typed(PatType { attrs, pat, ty, .. }) = arg {
            for attr in attrs {
                let Some(a) = parse_transformer_attr(attr.clone()) else {
                    continue;
                };

                let binding = match &**pat {
                    Pat::Ident(id) => &id.ident,
                    _ => panic!("Invalid pattern binding"),
                };

                transformers.push(a.clone());

                let variant_ident = match &**ty {
                    syn::Type::Path(type_path) => {
                        let last_segment = type_path.path.segments.last().unwrap();

                        if last_segment.ident == "Option" {
                            if let syn::PathArguments::AngleBracketed(args) =
                                &last_segment.arguments
                            {
                                if let Some(syn::GenericArgument::Type(syn::Type::Path(
                                    inner_type_path,
                                ))) = args.args.first()
                                {
                                    format_ident!(
                                        "{}",
                                        inner_type_path.path.segments.last().unwrap().ident
                                    )
                                } else {
                                    panic!("Unsupported Option inner type")
                                }
                            } else {
                                panic!("Unsupported Option generic")
                            }
                        } else {
                            format_ident!("{}", last_segment.ident)
                        }
                    }
                    _ => panic!("Unsupported argument type"),
                };

                let variant_str = variant_ident.to_string();
                let binding_str = binding.to_string();
                let binding_exp = if let syn::Type::Path(type_path) = &**ty {
                    if type_path.path.segments.last().unwrap().ident == "Option" {
                        let inner_type = match &type_path.path.segments.last().unwrap().arguments {
                            syn::PathArguments::AngleBracketed(args) => args.args.first().unwrap(),
                            _ => panic!("Unsupported Option type"),
                        };
                        quote! {
                            let #binding: Option<#inner_type> = match args_iter.next() {
                                Some(arg) => match arg.contents {
                                    Some(CommandArgument::#variant_ident(inner)) => Some(inner),
                                    _ => None,
                                },
                                None => None,
                            };
                        }
                    } else {
                        quote! {
                            let #binding = {
                                let Some(Token { contents: Some(CommandArgument::#variant_ident(b)), .. }) = args_iter.next() else {
                                    return Box::pin(async move {
                                        Err(CommandError::arg_not_found(#binding_str, Some(#variant_str)))
                                    })
                                };
                                b
                            };
                        }
                    }
                } else {
                    quote! {
                        let #binding = {
                            let Some(Token { contents: Some(CommandArgument::#variant_ident(b)), .. }) = args_iter.next() else {
                                return Box::pin(async move {
                                    Err(CommandError::arg_not_found(#binding_str, Some(#variant_str)))
                                })
                            };
                            b
                        };
                    }
                };

                arg_bindings.push(binding_exp);
            }
            new_fn_args.push(arg.clone());
        } else {
            new_fn_args.push(arg.clone());
        }
    }

    let args_ident: syn::Ident = syn::parse_str("args").unwrap();
    let args_ty: syn::Type = syn::parse_str("Vec<Token>").unwrap();

    new_fn_args.push(FnArg::Typed(PatType {
        attrs: Vec::new(),
        pat: Box::new(Pat::Ident(syn::PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: args_ident,
            subpat: None,
        })),
        colon_token: Default::default(),
        ty: Box::new(args_ty),
    }));

    let transformer_fns: Vec<_> = transformers
        .iter()
        .map(|tr| {
            let ident = format_ident!("{}", tr);
            quote! { Arc::new(Transformers::#ident) }
        })
        .collect();

    let fn_name = &sig.ident;
    let fn_async = &sig.asyncness;
    let fn_output = &sig.output;
    let fn_generics = &sig.generics;
    let fn_where = &sig.generics.where_clause;

    let stmts = &block.stmts;

    let expanded = quote! {
        #vis #fn_async fn #fn_name #fn_generics (&'life0 self, ctx: Context, msg: Message, args: Vec<Token>) #fn_output #fn_where {
            let mut args_iter = args.clone().into_iter();
            #(#arg_bindings)*

            #(#stmts)*
        }

        fn get_transformers(&self) -> Vec<TransformerFn> {
            vec![ #(#transformer_fns),* ]
        }
    };

    TokenStream::from(expanded)
}

fn parse_transformer_attr(attr: Attribute) -> Option<String> {
    let mut segments_iter = attr.meta.path().segments.clone().into_iter();

    if segments_iter.next()?.ident != "transformers" {
        return None;
    }

    segments_iter.next().map(|s| s.ident.to_string())
}
