use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, Expr, FnArg, ItemFn, Lit, Meta, MetaNameValue,
    PatType, ReturnType, Token, Type,
};

/// 関数の入力パラメータ型を取得する
fn extract_input_type(args: &Punctuated<FnArg, Token![,]>) -> Option<&Type> {
    if args.len() != 1 {
        return None;
    }

    match &args[0] {
        FnArg::Typed(PatType { ty, .. }) => Some(ty),
        _ => None,
    }
}

/// 関数の戻り値型からResultの中身を取得する
fn extract_output_and_error_types(return_type: &ReturnType) -> Option<(&Type, &Type)> {
    match return_type {
        ReturnType::Type(_, ty) => {
            // Result<OutputType, ErrorType>の形式を期待
            if let Type::Path(type_path) = ty.as_ref() {
                let path = &type_path.path;
                if path.segments.len() == 1 && path.segments[0].ident == "Result" {
                    if let syn::PathArguments::AngleBracketed(args) = &path.segments[0].arguments {
                        if args.args.len() == 2 {
                            let output_type = match &args.args[0] {
                                syn::GenericArgument::Type(t) => t,
                                _ => return None,
                            };
                            let error_type = match &args.args[1] {
                                syn::GenericArgument::Type(t) => t,
                                _ => return None,
                            };
                            return Some((output_type, error_type));
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// #[stream_worker]マクロの実装
pub fn stream_worker_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 関数を解析
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;

    // 属性パラメータを解析
    let args = parse_macro_input!(attr with Punctuated<Meta, Token![,]>::parse_terminated);

    let mut durable_name = None;

    for meta in args {
        if let Meta::NameValue(MetaNameValue { path, value, .. }) = meta {
            if path.is_ident("durable") {
                if let Expr::Lit(expr_lit) = &value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        durable_name = Some(lit_str.value());
                    }
                }
            }
        }
    }

    // 関数の入力型と出力型を取得
    let input_type = match extract_input_type(&input_fn.sig.inputs) {
        Some(ty) => ty,
        None => {
            return syn::Error::new(
                Span::call_site(),
                "Function must have exactly one parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    let (output_type, error_type) = match extract_output_and_error_types(&input_fn.sig.output) {
        Some((output, error)) => (output, error),
        None => {
            return syn::Error::new(
                Span::call_site(),
                "Function must return Result<OutputType, ErrorType>",
            )
            .to_compile_error()
            .into();
        }
    };

    // durableメソッドの呼び出しを生成
    let durable_method = if let Some(name) = durable_name {
        let name_lit = syn::LitStr::new(&name, Span::call_site());
        quote! { .durable(#name_lit) }
    } else {
        quote! { .durable_auto() }
    };

    // StreamWorkerを構築して実行するコードを生成
    let expanded = quote! {
        // 元の関数をそのまま保持
        #input_fn

        // ワーカーを実行する関数を生成
        #fn_vis async fn #fn_name _worker(
            js_ctx: &shared_infra::jetstream::JetStreamCtx,
            shutdown: tokio_util::sync::CancellationToken
        ) -> anyhow::Result<()> {
            use std::sync::Arc;
            use shared_core::event_metadata::HasStreamDef;
            use shared_core::stream_worker::StreamWorker;
            use shared_infra::jetstream::{JsPublisher, JsSubscriber};

            // サブスクライバーとパブリッシャーを作成
            let subscriber = Arc::new(JsSubscriber::<#input_type>::new(
                js_ctx.clone(),
                <#input_type as HasStreamDef>::stream_name(),
                <#input_type as HasStreamDef>::stream_subject(),
            ));

            let publisher = Arc::new(JsPublisher::<#output_type>::new(
                js_ctx.clone(),
                <#output_type as HasStreamDef>::stream_name(),
                <#output_type as HasStreamDef>::stream_subject(),
            ));

            // ハンドラ関数をラップ
            let handler = |event: #input_type| -> futures::future::BoxFuture<'static, std::result::Result<#output_type, #error_type>> {
                Box::pin(async move {
                    #fn_name(event).await
                })
            };

            // StreamWorkerを構築して実行
            StreamWorker::new(subscriber, publisher, handler)
                #durable_method
                .run(shutdown)
                .await
                .map_err(|e| anyhow::anyhow!("Worker error: {}", e))
        }
    };

    TokenStream::from(expanded)
}
