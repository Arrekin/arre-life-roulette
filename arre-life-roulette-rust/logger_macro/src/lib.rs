

extern crate proc_macro;
use proc_macro::{ TokenStream};
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn, ReturnType, Type, PathArguments, GenericArgument};
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[proc_macro_attribute]
pub fn duplicate_method(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let name = &input.sig.ident;
    let body = &input.block;
    let duplicated_name = Ident::new(&format!("_{}", name), name.span());
    let signature_args = &input.sig.inputs;
    let args = input.sig.inputs.iter().skip(1).collect::<Punctuated<_, Comma>>();
    let attrs = input.attrs.iter().cloned().map(|attr| quote!(#attr));
    let attrs_size = input.attrs.len();
    let ret_type = match &input.sig.output {
        ReturnType::Type(_, t) => t,
        _ => panic!("Function must return a value"),
    };

    // let inner_type = match &**ret_type {
    //     Type::Path(path) => {
    //         let segment = path.path.segments.last().unwrap();
    //         if let PathArguments::AngleBracketed(bracketed_args) = &segment.arguments {
    //             let type_arg = &bracketed_args.args[0];
    //             if let GenericArgument::Type(ty) = type_arg {
    //                 Some(ty.clone())
    //             } else {
    //                 None
    //             }
    //         } else {
    //             None
    //         }
    //     },
    //     _ => None,
    // };
    // let inner_type = inner_type.unwrap_or_else(|| panic!("Could not extract inner type from ArreError"));

    let expanded = quote! {
        #[allow(dead_code)]
        fn #duplicated_name(#signature_args) -> ArreResult<#ret_type> {
            #body
        }

        #[allow(dead_code)]
        log_syntax!{
            #attrs_size aaa
            #(#attrs)*
            fn #duplicated_name(#signature_args) -> ArreResult<#ret_type> {
            #body
        }}
        fn #name(#signature_args) -> #ret_type {
            match self.#duplicated_name(#args) {
                Ok(value) => value,
                Err(e) => {
                    log_error(&e);
                    <#ret_type>::default()
                }
            }
        }
    };
    TokenStream::from(expanded)
}