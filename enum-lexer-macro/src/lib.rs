

#![feature(proc_macro_quote)]

mod ast;
mod gen;
use syn;
use ast::EnumLexer;
use gen::generate;
use proc_macro::TokenStream;
use quote::quote;

// use std::process::Command;
use proc_macro2::Span;

#[proc_macro]
pub fn enum_lexer(input: TokenStream) -> TokenStream {
    enum_lexer_optional_bool(input, false)
}

#[proc_macro]
pub fn enum_lexer_test(input: TokenStream) -> TokenStream {
    enum_lexer_optional_bool(input, true)
}


fn enum_lexer_optional_bool(input: TokenStream, test: bool) -> TokenStream {
    let lexer = syn::parse_macro_input!(input as EnumLexer);
    match generate(lexer, test) {
        Ok(stream) => {
            // let mut file = std::fs::File::create("gen_lexer.rs").unwrap();
            // use std::io::Write;

            // write!(file, "#![feature(exclusive_range_pattern)]\n").unwrap();
            // write!(file, "{}", stream).unwrap();

            // Command::new("rustfmt")
            //     .arg("gen_lexer.rs")
            //     .spawn().unwrap();

            stream
        }
        Err(e) => {
            let what = syn::LitStr::new(format!("{}", e).as_str(), Span::call_site());

            quote!{
                compile_error!(#what)
            }
        }
    }.into()
}

// #[proc_macro]
// pub fn get_dfa_num(input: TokenStream) -> TokenStream {
//     let regex = syn::parse_macro_input!(input as syn::LitStr);
//     let regex = regex.value();
//     if let Some(i) = gen::get_dfa_num(&regex) {
//         quote! { #i }
//     } else {
//         quote! { compile_error!("regex not found in lexer.") }
//     }.into()
// }