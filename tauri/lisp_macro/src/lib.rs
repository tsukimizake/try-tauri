use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn lisp_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    let expanded = quote! {
        #input

        inventory::submit! {
            crate::cadprims::LispPrimitive {
                name: #fn_name_str,
                func: #fn_name
            }
        }
    };

    TokenStream::from(expanded)
}
