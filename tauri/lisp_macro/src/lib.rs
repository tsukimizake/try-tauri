use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, ItemFn, LitStr};

struct LispFnArgs {
    name: Option<String>,
}

impl Parse for LispFnArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(LispFnArgs { name: None });
        }
        let name_lit: LitStr = input.parse()?;
        Ok(LispFnArgs {
            name: Some(name_lit.value()),
        })
    }
}

/// Registers a function as a Lisp primitive that will be available in the Lisp environment.
/// Arguments to the function will be automatically evaluated before being passed to the function.
///
/// # Examples
///
/// ```
/// // Using the function name as the Lisp name
/// #[lisp_fn]
/// fn add(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
///     // registers as "add"
/// }
///
/// // Using a custom name for the Lisp function
/// #[lisp_fn("+")]
/// fn add_op(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
///     // registers as "+"
/// }
/// ```
#[proc_macro_attribute]
pub fn lisp_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as LispFnArgs);
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = args.name.unwrap_or_else(|| fn_name.to_string());

    let expanded = quote! {
        #input

        inventory::submit! {
            LispPrimitive {
                name: #fn_name_str,
                func: #fn_name
            }
        }
    };

    TokenStream::from(expanded)
}

/// Registers a function as a Lisp special form that will be available in the Lisp environment.
/// Arguments to the special form will NOT be automatically evaluated before being passed to the function.
///
/// # Examples
///
/// ```
/// // Using the function name as the Lisp name
/// #[lisp_sp_form]
/// fn my_if(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
///     // registers as "my_if"
/// }
///
/// // Using a custom name for the special form
/// #[lisp_sp_form("if-let")]
/// fn if_let(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
///     // registers as "if-let"
/// }
/// ```
#[proc_macro_attribute]
pub fn lisp_sp_form(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as LispFnArgs);
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_name_str = args.name.unwrap_or_else(|| fn_name.to_string());

    let expanded = quote! {
        #input

        inventory::submit! {
            LispSpecialForm {
                name: #fn_name_str,
                func: #fn_name
            }
        }
    };

    TokenStream::from(expanded)
}
