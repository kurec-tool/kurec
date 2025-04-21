use heck::ToKebabCase;
use inflections::case::is_kebab_case;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Expr, Ident, Lit, Meta,
    MetaNameValue, Token,
};

pub fn stream_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 元の型定義を解析
    let input = parse_macro_input!(item as DeriveInput);
    let type_name = &input.ident;

    // 属性パラメータを解析
    let args = parse_macro_input!(attr with Punctuated<Meta, Token![,]>::parse_terminated);

    let mut subjects: Option<Vec<String>> = None;
    let mut retention: Option<String> = None;
    let mut max_consumers: Option<u32> = None;
    let mut max_msgs: Option<u64> = None;
    let mut max_bytes: Option<u64> = None;
    let mut max_age: Option<String> = None;
    let mut max_msg_size: Option<u32> = None;
    let mut storage: Option<String> = None;
    let mut discard: Option<String> = None;
    let mut duplicate_window: Option<String> = None;
    let mut allow_rollup: Option<bool> = None;
    let mut deny_delete: Option<bool> = None;
    let mut deny_purge: Option<bool> = None;
    let mut description: Option<String> = None;

    for meta in args {
        if let Meta::NameValue(MetaNameValue { path, value, .. }) = meta {
            match path.get_ident().map(|i| i.to_string()).as_deref() {
                Some("subjects") => {
                    // 配列リテラルの解析
                    // 簡略化のため省略
                }
                Some("retention") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            retention = Some(lit_str.value());
                        }
                    }
                }
                Some("max_consumers") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Int(lit_int) = &expr_lit.lit {
                            max_consumers = lit_int.base10_parse().ok();
                        }
                    }
                }
                Some("max_msgs") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Int(lit_int) = &expr_lit.lit {
                            max_msgs = lit_int.base10_parse().ok();
                        }
                    }
                }
                Some("max_bytes") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Int(lit_int) = &expr_lit.lit {
                            max_bytes = lit_int.base10_parse().ok();
                        }
                    }
                }
                Some("max_age") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            // humantime形式の検証
                            let duration_str = lit_str.value();
                            if humantime::parse_duration(&duration_str).is_err() {
                                return syn::Error::new(
                                    lit_str.span(),
                                    format!("Invalid duration format: '{}'. Expected format like '7d', '24h', etc.", duration_str),
                                )
                                .to_compile_error()
                                .into();
                            }
                            max_age = Some(duration_str);
                        }
                    }
                }
                Some("max_msg_size") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Int(lit_int) = &expr_lit.lit {
                            max_msg_size = lit_int.base10_parse().ok();
                        }
                    }
                }
                Some("storage") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            storage = Some(lit_str.value());
                        }
                    }
                }
                Some("discard") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            discard = Some(lit_str.value());
                        }
                    }
                }
                Some("duplicate_window") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            // humantime形式の検証
                            let duration_str = lit_str.value();
                            if humantime::parse_duration(&duration_str).is_err() {
                                return syn::Error::new(
                                    lit_str.span(),
                                    format!("Invalid duration format: '{}'. Expected format like '2m', '1h', etc.", duration_str),
                                )
                                .to_compile_error()
                                .into();
                            }
                            duplicate_window = Some(duration_str);
                        }
                    }
                }
                Some("allow_rollup") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Bool(lit_bool) = &expr_lit.lit {
                            allow_rollup = Some(lit_bool.value);
                        }
                    }
                }
                Some("deny_delete") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Bool(lit_bool) = &expr_lit.lit {
                            deny_delete = Some(lit_bool.value);
                        }
                    }
                }
                Some("deny_purge") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Bool(lit_bool) = &expr_lit.lit {
                            deny_purge = Some(lit_bool.value);
                        }
                    }
                }
                Some("description") => {
                    if let Expr::Lit(expr_lit) = &value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            description = Some(lit_str.value());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // ストリーム名は型名をケバブケースに変換
    let name = type_name.to_string().to_kebab_case();

    // 各設定項目の式を生成
    let max_consumers_expr = if let Some(value) = max_consumers {
        quote! { config.max_consumers = Some(#value); }
    } else {
        quote! {}
    };

    let max_msgs_expr = if let Some(value) = max_msgs {
        quote! { config.max_msgs = Some(#value); }
    } else {
        quote! {}
    };

    let max_bytes_expr = if let Some(value) = max_bytes {
        quote! { config.max_bytes = Some(#value); }
    } else {
        quote! {}
    };

    let max_age_expr = if let Some(value) = &max_age {
        quote! { config.max_age = Some(humantime::parse_duration(#value).unwrap()); }
    } else {
        quote! {}
    };

    let max_msg_size_expr = if let Some(value) = max_msg_size {
        quote! { config.max_msg_size = Some(#value); }
    } else {
        quote! {}
    };

    let storage_expr = match storage.as_deref() {
        Some("file") => {
            quote! { config.storage = Some(shared_types::stream::StorageType::File); }
        }
        Some("memory") => {
            quote! { config.storage = Some(shared_types::stream::StorageType::Memory); }
        }
        Some(invalid) => {
            return syn::Error::new(
                Span::call_site(),
                format!(
                    "Invalid storage type: '{}'. Expected 'file' or 'memory'.",
                    invalid
                ),
            )
            .to_compile_error()
            .into();
        }
        None => quote! {},
    };

    let discard_expr = match discard.as_deref() {
        Some("old") => {
            quote! { config.discard = Some(shared_types::stream::DiscardPolicy::Old); }
        }
        Some("new") => {
            quote! { config.discard = Some(shared_types::stream::DiscardPolicy::New); }
        }
        Some(invalid) => {
            return syn::Error::new(
                Span::call_site(),
                format!(
                    "Invalid discard policy: '{}'. Expected 'old' or 'new'.",
                    invalid
                ),
            )
            .to_compile_error()
            .into();
        }
        None => quote! {},
    };

    let duplicate_window_expr = if let Some(value) = &duplicate_window {
        quote! { config.duplicate_window = Some(humantime::parse_duration(#value).unwrap()); }
    } else {
        quote! {}
    };

    let allow_rollup_expr = if let Some(value) = allow_rollup {
        quote! { config.allow_rollup = Some(#value); }
    } else {
        quote! {}
    };

    let deny_delete_expr = if let Some(value) = deny_delete {
        quote! { config.deny_delete = Some(#value); }
    } else {
        quote! {}
    };

    let deny_purge_expr = if let Some(value) = deny_purge {
        quote! { config.deny_purge = Some(#value); }
    } else {
        quote! {}
    };

    let description_expr = if let Some(value) = &description {
        quote! { config.description = Some(#value.to_string()); }
    } else {
        quote! {}
    };

    let retention_expr = match retention.as_deref() {
        Some("limits") => {
            quote! { config.retention = Some(shared_types::stream::RetentionPolicy::Limits); }
        }
        Some("interest") => {
            quote! { config.retention = Some(shared_types::stream::RetentionPolicy::Interest); }
        }
        Some("workqueue") => {
            quote! { config.retention = Some(shared_types::stream::RetentionPolicy::WorkQueue); }
        }
        Some(invalid) => {
            return syn::Error::new(
                Span::call_site(),
                format!("Invalid retention policy: '{}'. Expected 'limits', 'interest', or 'workqueue'.", invalid),
            )
            .to_compile_error()
            .into();
        }
        None => quote! {},
    };

    let expanded = quote! {
        #input

        impl shared_types::stream::Stream for #type_name {
            const NAME: &'static str = #name;

            fn config() -> shared_types::stream::StreamConfig {
                let mut config = shared_types::stream::StreamConfig {
                    name: #name.to_string(),
                    subjects: None,
                    retention: None,
                    max_consumers: None,
                    max_msgs: None,
                    max_bytes: None,
                    max_age: None,
                    max_msg_size: None,
                    storage: None,
                    discard: None,
                    duplicate_window: None,
                    allow_rollup: None,
                    deny_delete: None,
                    deny_purge: None,
                    description: None,
                };

                #retention_expr
                #max_consumers_expr
                #max_msgs_expr
                #max_bytes_expr
                #max_age_expr
                #max_msg_size_expr
                #storage_expr
                #discard_expr
                #duplicate_window_expr
                #allow_rollup_expr
                #deny_delete_expr
                #deny_purge_expr
                #description_expr

                config
            }
        }
    };

    TokenStream::from(expanded)
}
