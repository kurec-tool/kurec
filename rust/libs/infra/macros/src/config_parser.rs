use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned, // span() メソッドを利用するためにインポート
    Expr,
    Lit,
    LitBool,
    LitInt,
    LitStr,
    Meta,
    MetaNameValue,
    Result,
    Token,
};

use proc_macro2::TokenStream as TokenStream2; // quote 用
use quote::{quote, ToTokens};
// 重複した use syn::{...} ブロックを削除

/// マクロ属性のパラメータを保持する構造体 (解析用)
#[derive(Default, Debug)]
pub struct StreamConfigArgs {
    pub stream_name_override: Option<LitStr>,
    pub max_age: Option<LitStr>,
    pub max_msgs: Option<LitInt>,
    pub max_bytes: Option<LitInt>,
    pub max_msg_size: Option<LitInt>,
    pub storage: Option<LitStr>,   // "file" or "memory"
    pub retention: Option<LitStr>, // "limits", "interest", "workqueue"
    pub discard: Option<LitStr>,   // "old" or "new"
    pub duplicate_window: Option<LitStr>,
    pub allow_rollup: Option<LitBool>,
    pub deny_delete: Option<LitBool>,
    pub deny_purge: Option<LitBool>,
    pub description: Option<LitStr>,
    // 他の StreamConfig フィールドに対応するパラメータも追加可能
}

/// マクロが生成する定数の型となる構造体
#[derive(Debug)]
pub struct StreamAttributes {
    pub max_age: Option<LitStr>,
    pub max_msgs: Option<LitInt>,
    pub max_bytes: Option<LitInt>,
    pub max_msg_size: Option<LitInt>,
    pub storage: Option<LitStr>,
    pub retention: Option<LitStr>,
    pub discard: Option<LitStr>,
    pub duplicate_window: Option<LitStr>,
    pub allow_rollup: Option<LitBool>,
    pub deny_delete: Option<LitBool>,
    pub deny_purge: Option<LitBool>,
    pub description: Option<LitStr>,
}

// StreamConfigArgs から StreamAttributes を作成する From 実装
impl From<StreamConfigArgs> for StreamAttributes {
    fn from(args: StreamConfigArgs) -> Self {
        Self {
            max_age: args.max_age,
            max_msgs: args.max_msgs,
            max_bytes: args.max_bytes,
            max_msg_size: args.max_msg_size,
            storage: args.storage,
            retention: args.retention,
            discard: args.discard,
            duplicate_window: args.duplicate_window,
            allow_rollup: args.allow_rollup,
            deny_delete: args.deny_delete,
            deny_purge: args.deny_purge,
            description: args.description,
        }
    }
}

// StreamAttributes を TokenStream2 に変換してマクロで使えるようにする
impl ToTokens for StreamAttributes {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let max_age = self
            .max_age
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let max_msgs = self
            .max_msgs
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let max_bytes = self
            .max_bytes
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let max_msg_size = self
            .max_msg_size
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let storage = self
            .storage
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let retention = self
            .retention
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let discard = self
            .discard
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let duplicate_window = self
            .duplicate_window
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let allow_rollup = self
            .allow_rollup
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let deny_delete = self
            .deny_delete
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let deny_purge = self
            .deny_purge
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));
        let description = self
            .description
            .as_ref()
            .map_or(quote!(None), |l| quote!(Some(#l)));

        // StreamAttributes 構造体の初期化コードを生成
        let expanded = quote! {
            ::infra_macros::StreamAttributes { // パス修正
                max_age: #max_age,
                max_msgs: #max_msgs,
                max_bytes: #max_bytes,
                max_msg_size: #max_msg_size,
                storage: #storage,
                retention: #retention,
                discard: #discard,
                duplicate_window: #duplicate_window,
                allow_rollup: #allow_rollup,
                deny_delete: #deny_delete,
                deny_purge: #deny_purge,
                description: #description,
            }
        };
        expanded.to_tokens(tokens);
    }
}

impl Parse for StreamConfigArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut args = StreamConfigArgs::default();

        for meta in metas {
            if let Meta::NameValue(MetaNameValue { path, value, .. }) = meta {
                let ident = path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new(path.span(), "Expected identifier"))?;
                match ident.to_string().as_str() {
                    "stream" => {
                        if let Expr::Lit(expr_lit) = &value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                args.stream_name_override = Some(lit_str.clone());
                            } else {
                                return Err(syn::Error::new(
                                    value.span(),
                                    "Expected string literal for 'stream'",
                                ));
                            }
                        } else {
                            return Err(syn::Error::new(
                                value.span(),
                                "Expected literal for 'stream'",
                            ));
                        }
                    }
                    "max_age" => args.max_age = parse_lit_str(value)?,
                    "max_msgs" => args.max_msgs = parse_lit_int(value)?,
                    "max_bytes" => args.max_bytes = parse_lit_int(value)?,
                    "max_msg_size" => args.max_msg_size = parse_lit_int(value)?,
                    "storage" => args.storage = parse_lit_str(value)?,
                    "retention" => args.retention = parse_lit_str(value)?,
                    "discard" => args.discard = parse_lit_str(value)?,
                    "duplicate_window" => args.duplicate_window = parse_lit_str(value)?,
                    "allow_rollup" => args.allow_rollup = parse_lit_bool(value)?,
                    "deny_delete" => args.deny_delete = parse_lit_bool(value)?,
                    "deny_purge" => args.deny_purge = parse_lit_bool(value)?,
                    "description" => args.description = parse_lit_str(value)?,
                    _ => {
                        return Err(syn::Error::new(
                            path.span(),
                            format!("Unknown attribute key: {}", ident),
                        ));
                    }
                }
            } else {
                return Err(syn::Error::new(
                    meta.span(),
                    "Expected name-value attribute",
                ));
            }
        }
        Ok(args)
    }
}

// ヘルパー関数群
fn parse_lit_str(value: Expr) -> Result<Option<LitStr>> {
    if let Expr::Lit(expr_lit) = value {
        if let Lit::Str(lit_str) = expr_lit.lit {
            Ok(Some(lit_str))
        } else {
            Err(syn::Error::new(expr_lit.span(), "Expected string literal"))
        }
    } else {
        Err(syn::Error::new(value.span(), "Expected literal"))
    }
}

fn parse_lit_int(value: Expr) -> Result<Option<LitInt>> {
    if let Expr::Lit(expr_lit) = value {
        if let Lit::Int(lit_int) = expr_lit.lit {
            Ok(Some(lit_int))
        } else {
            Err(syn::Error::new(expr_lit.span(), "Expected integer literal"))
        }
    } else {
        Err(syn::Error::new(value.span(), "Expected literal"))
    }
}

fn parse_lit_bool(value: Expr) -> Result<Option<LitBool>> {
    if let Expr::Lit(expr_lit) = value {
        if let Lit::Bool(lit_bool) = expr_lit.lit {
            Ok(Some(lit_bool))
        } else {
            Err(syn::Error::new(expr_lit.span(), "Expected boolean literal"))
        }
    } else {
        Err(syn::Error::new(value.span(), "Expected literal"))
    }
}
