use heck::CamelCase;
use proc_macro::TokenStream;
use proc_macro_error::{diagnostic, Diagnostic, Level::Error};
use quote::{format_ident, quote};
use syn::{spanned::Spanned, FnArg, Ident, Item, ItemMod, Pat, ReturnType, Visibility};

mod signature;
mod simple_arg;

use signature::Signature;
use simple_arg::SimpleArg;

#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn defunctionalize(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut mod_item = syn::parse_macro_input!(item as ItemMod);
    let signature = syn::parse_macro_input!(attr as Signature);

    let items = match &mod_item.content {
        Some((.., items)) => items,
        None => panic!(),
    };

    let derive_position = mod_item
        .attrs
        .iter()
        .position(|attr| attr.path.segments[0].ident == "derive");
    let derives = match derive_position {
        Some(position) => vec![mod_item.attrs.remove(position)],
        None => vec![],
    };

    let mod_name = &mod_item.ident;
    let enum_name = signature
        .ident
        .clone()
        .unwrap_or_else(|| format_ident!("{}", mod_name.to_string().to_camel_case()));

    let functions = items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(item) => Some(item),
            _ => None,
        })
        .filter(|item| matches!(item.vis, Visibility::Public(..)))
        .collect::<Vec<_>>();

    let case_names = functions
        .iter()
        .map(|item| item.sig.ident.to_string().to_camel_case())
        .map(|name| format_ident!("{}", name))
        .collect::<Vec<_>>();

    let function_names = functions
        .iter()
        .map(|item| &item.sig.ident)
        .collect::<Vec<_>>();

    let case_arg_names = functions
        .iter()
        .map(|item| {
            item.sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    FnArg::Receiver(..) => Err(diagnostic!(
                        arg.span(),
                        Error,
                        "defunctionalized functions cannot have receivers"
                    )),
                    FnArg::Typed(pat) => Ok(pat.pat.as_ref()),
                })
                .map(|pat| match pat? {
                    Pat::Ident(ident) => Ok(&ident.ident),
                    pat => Err(diagnostic!(
                        pat.span(),
                        Error,
                        "arguments to defunctionalized functions must be named"
                    )),
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .map(|mut args| {
            match &mut args {
                Ok(args) => args.truncate(args.len() - signature.inputs.len()),
                Err(..) => {}
            }
            args
        })
        .map(|args| {
            let args = args?;
            Ok(if args.is_empty() { vec![] } else { vec![args] })
        })
        .collect::<Result<Vec<_>, Diagnostic>>();
    let case_arg_names = match case_arg_names {
        Ok(case_arg_names) => case_arg_names,
        Err(diagnostic) => diagnostic.abort(),
    };

    let case_arg_types = functions
        .iter()
        .map(|item| {
            item.sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    FnArg::Receiver(..) => unreachable!(),
                    FnArg::Typed(pat) => pat.ty.as_ref(),
                })
                .collect::<Vec<_>>()
        })
        .map(|mut args| {
            args.truncate(args.len() - signature.inputs.len());
            args
        })
        .map(|args| if args.is_empty() { vec![] } else { vec![args] })
        .collect::<Vec<_>>();

    let visibility = &mod_item.vis;
    let generics = &signature.generics;
    let where_clause = &signature.generics.where_clause;
    let inputs = &signature.inputs;
    let input_types = inputs.iter().map(|arg| &arg.ty).collect::<Vec<_>>();
    let input_names = &signature
        .inputs
        .iter()
        .map(|arg| &arg.ident)
        .collect::<Vec<&Ident>>();
    let arg_idents = std::iter::repeat(&input_names);
    let output = &signature.output;
    let output_type = match output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(.., ty) => quote!(#ty),
    };

    let output = quote! {
        #mod_item

        #(#derives)*
        #visibility enum #enum_name {
            #(#case_names#((#(#case_arg_types),*))*),*
        }

        impl #generics defunctionalize::DeFn<(#(#input_types),*)> for #enum_name #where_clause {
            type Output = #output_type;

            fn call (self, (#(#input_names),*): (#(#input_types),*)) #output {
                self.call(#(#input_names),*)
            }
        }

        impl #enum_name {
            #visibility fn call #generics (self, #inputs) #output #where_clause {
                match self {
                    #(Self::#case_names#((#(#case_arg_names),*))* => {
                        #mod_name::#function_names(
                            #(#(#case_arg_names,)*)*
                            #(#arg_idents),*
                        )
                    })*
                }
            }
        }
    };

    output.into()
}
