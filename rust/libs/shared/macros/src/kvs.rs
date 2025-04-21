//! KVSバケット定義マクロの実装

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    // meta::NestedMeta, // meta::NestedMeta のインポートを削除
    parse::Parser, // Parser をインポート
    parse_macro_input,
    punctuated::Punctuated,
    DeriveInput,
    Expr,
    Lit,
    LitBool,
    LitInt,
    LitStr,
    Meta, // NestedMeta を削除し Meta を使う
    MetaNameValue,
    // NestedMeta, // 削除
    Path,
    Token,
};

// Helper function to parse literal to a specific type
fn parse_lit<T: syn::parse::Parse>(lit: &Lit, attr_name: &str) -> syn::Result<T> {
    let lit_str = match lit {
        Lit::Str(s) => s.value(),
        Lit::Int(i) => i.to_string(),
        Lit::Bool(b) => b.value().to_string(),
        _ => {
            return Err(syn::Error::new_spanned(
                lit,
                format!("Invalid literal type for {}", attr_name),
            ))
        }
    };
    syn::parse_str::<T>(&lit_str)
        .map_err(|e| syn::Error::new_spanned(lit, format!("Failed to parse {}: {}", attr_name, e)))
}

// Helper function to parse string literal
fn parse_lit_str(lit: &Lit, attr_name: &str) -> syn::Result<String> {
    match lit {
        Lit::Str(s) => Ok(s.value()),
        _ => Err(syn::Error::new_spanned(
            lit,
            format!("{} must be a string literal", attr_name),
        )),
    }
}

// Helper function to parse integer literal
fn parse_lit_int<T: std::str::FromStr>(lit: &Lit, attr_name: &str) -> syn::Result<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match lit {
        Lit::Int(i) => i.base10_parse::<T>().map_err(|e| {
            syn::Error::new_spanned(
                lit,
                format!("Failed to parse {} as integer: {}", attr_name, e),
            )
        }),
        _ => Err(syn::Error::new_spanned(
            lit,
            format!("{} must be an integer literal", attr_name),
        )),
    }
}

// Helper function to parse boolean literal
fn parse_lit_bool(lit: &Lit, attr_name: &str) -> syn::Result<bool> {
    match lit {
        Lit::Bool(b) => Ok(b.value()),
        _ => Err(syn::Error::new_spanned(
            lit,
            format!("{} must be a boolean literal", attr_name),
        )),
    }
}

// Helper function to parse array of string literals
fn parse_lit_str_array(lit: &Lit, attr_name: &str) -> syn::Result<Vec<String>> {
    match lit {
        Lit::Verbatim(v) => {
            // Try parsing as an array expression: `["tag1", "tag2"]`
            let expr: Expr = syn::parse_str(&v.to_string())?;
            if let Expr::Array(expr_array) = expr {
                let mut tags = Vec::new();
                for elem in expr_array.elems {
                    if let Expr::Lit(expr_lit) = elem {
                        if let Lit::Str(lit_str) = expr_lit.lit {
                            tags.push(lit_str.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                expr_lit.lit,
                                format!("Elements of {} must be string literals", attr_name),
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            elem,
                            format!("Elements of {} must be literals", attr_name),
                        ));
                    }
                }
                Ok(tags)
            } else {
                Err(syn::Error::new_spanned(
                    lit,
                    format!(
                        "{} must be an array literal like [\"tag1\", \"tag2\"]",
                        attr_name
                    ),
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            lit,
            format!(
                "{} must be an array literal like [\"tag1\", \"tag2\"]",
                attr_name
            ),
        )),
    }
}

pub fn kvs_bucket_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let type_name = &input.ident;
    // Meta のリストとしてパースする
    let attr_args = match Punctuated::<Meta, Token![,]>::parse_terminated.parse(attr) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    // --- Attribute Parsing ---
    let mut bucket_name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut max_value_size: Option<i32> = None;
    let mut history: Option<i64> = None;
    let mut max_age: Option<String> = None; // Store as string for humantime parsing later
    let mut max_bytes: Option<i64> = None;
    let mut storage: Option<String> = None; // Store as string for validation later
    let mut num_replicas: Option<usize> = None;
    let mut mirror_direct: Option<bool> = None;
    let mut compression: Option<bool> = None;
    let mut placement_cluster: Option<String> = None;
    let mut placement_tags: Option<Vec<String>> = None;

    for meta in attr_args {
        // Meta::NameValue を直接処理
        if let Meta::NameValue(MetaNameValue { path, value, .. }) = meta {
            if let Some(ident) = path.get_ident() {
                let ident_str = ident.to_string();
                // value は Expr 型なので、Lit に変換してから処理
                if let Expr::Lit(expr_lit) = value {
                    let lit = &expr_lit.lit; // Lit を取得
                    match ident_str.as_str() {
                        "bucket_name" => match parse_lit_str(lit, "bucket_name") {
                            Ok(val) => bucket_name = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "description" => match parse_lit_str(lit, "description") {
                            Ok(val) => description = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "max_value_size" => match parse_lit_int::<i32>(lit, "max_value_size") {
                            Ok(val) => max_value_size = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "history" => match parse_lit_int::<i64>(lit, "history") {
                            Ok(val) => history = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "max_age" => match parse_lit_str(lit, "max_age") {
                            Ok(val) => {
                                // Validate humantime format here
                                if ::humantime::parse_duration(&val).is_err() {
                                    // humantime を完全修飾
                                    return syn::Error::new_spanned(
                                        lit,
                                        format!("Invalid duration format for max_age: '{}'. Expected format like '7d', '24h', etc.", val),
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                                max_age = Some(val);
                            }
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "max_bytes" => match parse_lit_int::<i64>(lit, "max_bytes") {
                            Ok(val) => max_bytes = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "storage" => match parse_lit_str(lit, "storage") {
                            Ok(val) => {
                                // Validate storage type string here
                                if val != "file" && val != "memory" {
                                    return syn::Error::new_spanned(
                                        lit,
                                        format!(
                                            "Invalid storage type: '{}'. Expected 'file' or 'memory'.",
                                            val
                                        ),
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                                storage = Some(val);
                            }
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "num_replicas" => match parse_lit_int::<usize>(lit, "num_replicas") {
                            Ok(val) => num_replicas = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "mirror_direct" => match parse_lit_bool(lit, "mirror_direct") {
                            Ok(val) => mirror_direct = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "compression" => match parse_lit_bool(lit, "compression") {
                            Ok(val) => compression = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "placement_cluster" => match parse_lit_str(lit, "placement_cluster") {
                            Ok(val) => placement_cluster = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        "placement_tags" => match parse_lit_str_array(lit, "placement_tags") {
                            Ok(val) => placement_tags = Some(val),
                            Err(e) => return e.to_compile_error().into(),
                        },
                        _ => {
                            // Optionally return an error for unknown attributes
                            // return syn::Error::new_spanned(path, "Unknown attribute").to_compile_error().into();
                        }
                    }
                } else {
                    return syn::Error::new_spanned(value, "Attribute value must be a literal")
                        .to_compile_error()
                        .into();
                }
            } else {
                // Handle cases where path is not a simple ident if necessary
                return syn::Error::new_spanned(path, "Expected identifier for attribute name")
                    .to_compile_error()
                    .into();
            }
        } else {
            // NameValue 以外の Meta (例: #[attr(path)]) はエラーとするか無視するか
            // ここではエラーとする
            return syn::Error::new_spanned(
                meta,
                "Expected key-value attribute like `key = \"value\"`",
            )
            .to_compile_error()
            .into();
        }
    }

    let bucket_name = match bucket_name {
        Some(name) => name,
        None => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing required attribute `bucket_name`",
            )
            .to_compile_error()
            .into();
        }
    };

    // --- Code Generation ---
    let description_expr =
        description.map(|val| quote! { config.description = Some(#val.to_string()); });
    let max_value_size_expr = max_value_size.map(|val| quote! { config.max_value_size = #val; });
    let history_expr = history.map(|val| quote! { config.history = #val; });
    // humantime を完全修飾パスで呼び出す
    let max_age_expr = max_age.map(|val| {
        quote! { config.max_age = ::humantime::parse_duration(#val).expect("Duration already validated"); }
    });
    let max_bytes_expr = max_bytes.map(|val| quote! { config.max_bytes = #val; });
    // StorageType は stream モジュールにあると仮定して修正
    let storage_expr = storage.map(|val| match val.as_str() {
        "file" => quote! { config.storage = async_nats::jetstream::stream::StorageType::File; },
        "memory" => quote! { config.storage = async_nats::jetstream::stream::StorageType::Memory; },
        _ => unreachable!("Storage type already validated"), // Should not happen due to earlier validation
    });
    let num_replicas_expr = num_replicas.map(|val| quote! { config.num_replicas = #val; });
    let mirror_direct_expr = mirror_direct.map(|val| quote! { config.mirror_direct = #val; });
    let compression_expr = compression.map(|val| quote! { config.compression = #val; });

    // Placement needs special handling
    let placement_expr = if placement_cluster.is_some() || placement_tags.is_some() {
        let cluster_expr = placement_cluster
            .map(|c| quote! { cluster: Some(#c.to_string()), })
            .unwrap_or_else(|| quote! { cluster: None, });
        let tags_expr = placement_tags
            .map(|t| quote! { tags: Some(vec![#(#t.to_string()),*]), })
            .unwrap_or_else(|| quote! { tags: None, });
        quote! {
            // Placement 構造体は stream モジュールにあると仮定
            config.placement = Some(async_nats::jetstream::stream::Placement {
                #cluster_expr
                #tags_expr
            });
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #input

        impl shared_types::kvs::KvsBucket for #type_name {
            const BUCKET_NAME: &'static str = #bucket_name;

            fn config() -> async_nats::jetstream::kv::Config {
                // Use kv::Config::default() which might be more robust if async_nats changes defaults
                let mut config = async_nats::jetstream::kv::Config {
                    bucket: #bucket_name.to_string(),
                    ..Default::default() // Start with async_nats defaults
                };

                // Apply settings from attributes
                #description_expr
                #max_value_size_expr
                #history_expr
                #max_age_expr
                #max_bytes_expr
                #storage_expr
                #num_replicas_expr
                #mirror_direct_expr
                #compression_expr
                #placement_expr

                config
            }
        }
    };

    TokenStream::from(expanded)
}
