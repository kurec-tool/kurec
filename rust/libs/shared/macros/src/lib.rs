use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::time::Duration;
use syn::{
    parse_macro_input, punctuated::Punctuated, DeriveInput, Expr, Lit, LitStr, Meta, MetaNameValue,
    Token,
};

fn parse_duration(opt: &Option<String>) -> Option<Duration> {
    opt.as_ref().map(|s| {
        humantime::parse_duration(s)
            .unwrap_or_else(|e| panic!("invalid duration literal `{}`: {}", s, e))
    })
}

#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    // --- parse original item ------------------------------------------------
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // --- parse attribute args ------------------------------------------------
    let args = parse_macro_input!(attr with Punctuated<Meta, Token![,]>::parse_terminated);

    let mut stream: Option<String> = None;
    let mut subject: Option<String> = None;
    let mut max_age: Option<Duration> = None;
    let mut max_deliver: Option<u32> = None;
    let mut ack_wait: Option<Duration> = None;

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
                (Some("subject"), Expr::Lit(s)) => {
                    if let Lit::Str(lit_str) = &s.lit {
                        subject = Some(lit_str.value());
                    }
                }
                (Some("max_age"), Expr::Lit(s)) => {
                    if let Lit::Str(lit_str) = &s.lit {
                        max_age = parse_duration(&Some(lit_str.value()));
                    }
                }
                (Some("max_deliver"), Expr::Lit(s)) => {
                    if let Lit::Int(lit_int) = &s.lit {
                        max_deliver = Some(lit_int.base10_parse::<u32>().unwrap());
                    }
                }
                (Some("ack_wait"), Expr::Lit(s)) => {
                    if let Lit::Str(lit_str) = &s.lit {
                        ack_wait = parse_duration(&Some(lit_str.value()));
                    }
                }
                _ => {}
            }
        }
    }

    let stream = stream.expect("stream is required");
    let subject = subject.expect("subject is required");

    // Turn strings into literal tokens so quote! gets correct Expr
    let stream_lit = LitStr::new(&stream, Span::call_site());
    let subject_lit = LitStr::new(&subject, Span::call_site());

    // helper to build Option literals
    let opt_u32 = |opt: &Option<u32>| {
        if let Some(v) = opt {
            quote!(Some(#v))
        } else {
            quote!(None)
        }
    };

    // helper to build Option<Duration> literal tokens
    let opt_duration = |opt: &Option<Duration>| {
        if let Some(d) = opt {
            let secs = d.as_secs();
            let nanos = d.subsec_nanos();
            quote!(Some(std::time::Duration::new(#secs, #nanos)))
        } else {
            quote!(None)
        }
    };

    let max_age_ts = opt_duration(&max_age);
    let ack_wait_ts = opt_duration(&ack_wait);
    let max_deliver_ts = opt_u32(&max_deliver);

    // --- generated code ------------------------------------------------------
    let expanded = quote! {
        #[derive(Debug)]
        #input

        shared_core::event_metadata::inventory::submit! {
            shared_core::event_metadata::StreamDef {
                name: #stream_lit,
                subjects: &[#subject_lit],
                default_config: shared_core::event_metadata::StreamConfigDefaults {
                    max_age: #max_age_ts,
                    max_deliver: #max_deliver_ts,
                    ack_wait: #ack_wait_ts,
                }
            }
        }

        impl shared_core::event_metadata::HasStreamDef for #struct_name {
            fn stream_subject() -> &'static str { #subject_lit }
            fn stream_name() -> &'static str { #stream_lit }
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parse_duration() {
        assert_eq!(
            parse_duration(&Some("1s".to_string())),
            Some(Duration::new(1, 0))
        );
        assert_eq!(
            parse_duration(&Some("1m".to_string())),
            Some(Duration::new(60, 0))
        );
        assert_eq!(
            parse_duration(&Some("1h".to_string())),
            Some(Duration::new(3600, 0))
        );
        assert_eq!(parse_duration(&None), None);
    }
}
