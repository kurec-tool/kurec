use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, DeriveInput, Expr, Lit, LitStr, Meta, MetaNameValue,
    Token,
};

pub fn event_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // --- parse original item ------------------------------------------------
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // --- parse attribute args ------------------------------------------------
    let args = parse_macro_input!(attr with Punctuated<Meta, Token![,]>::parse_terminated);

    let mut stream: Option<String> = None;
    let mut event_name: Option<String> = None;

    for meta in args {
        if let Meta::NameValue(MetaNameValue {
            path, value: lit, ..
        }) = meta
        {
            match (path.get_ident().map(|i| i.to_string()).as_deref(), &lit) {
                (Some("stream"), Expr::Lit(s)) => {
                    if let Lit::Str(lit_str) = &s.lit {
                        stream = Some(lit_str.value());
                    }
                }
                (Some("event_name"), Expr::Lit(s)) => {
                    if let Lit::Str(lit_str) = &s.lit {
                        event_name = Some(lit_str.value());
                    }
                }
                _ => {}
            }
        }
    }

    let stream = stream.expect("stream parameter is required");
    let stream_lit = LitStr::new(&stream, Span::call_site());

    // イベント名が指定されていない場合は構造体名をsnake_caseに変換
    let event_name = event_name.unwrap_or_else(|| struct_name.to_string().to_snake_case());
    let event_name_lit = LitStr::new(&event_name, Span::call_site());

    // --- generated code ------------------------------------------------------
    let expanded = quote! {
        #input

        impl shared_core::event_metadata::Event for #struct_name {
            fn stream_name() -> &'static str {
                #stream_lit
            }

            fn event_name() -> &'static str {
                #event_name_lit
            }
        }
    };

    TokenStream::from(expanded)
}
