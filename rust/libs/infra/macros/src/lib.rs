use heck::ToKebabCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};

mod config_parser;
// pub use config_parser::StreamAttributes; // 削除: proc-macro クレートからは公開できない
use config_parser::{StreamAttributes, StreamConfigArgs};

/// イベント構造体に JetStream のストリーム名と設定属性を関連付けるマクロ。
///
/// このマクロは、対象の構造体に以下の関連定数を実装します:
/// - `STREAM_NAME: &'static str`: JetStream のストリーム名。
///   - デフォルト: 構造体名をケバブケース (`kebab-case`) に変換したもの。
///   - 属性 `stream = "..."` で上書き可能。
/// - `STREAM_ATTRIBUTES: StreamAttributes`: JetStream のストリーム設定属性。
///   - `infra_macros::config::StreamAttributes` 型。
///   - マクロ属性 (`max_age`, `storage` など) で指定された値を保持。
///
/// # 注意
/// このマクロを適用する構造体は `kurec_domain::event::Event` トレイトを実装している必要があります。
/// (現状、マクロ内でこの制約を直接チェックするのは難しいため、利用側の規約とします)
///
/// # 使用例
/// ```ignore
/// use infra_macros::define_event_stream;
/// use kurec_domain::event::Event;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// #[define_event_stream(max_age = "14d", storage = "file")]
/// pub struct MyEvent { /* ... */ }
/// impl Event for MyEvent {} // Event トレイトの実装が必要
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// #[define_event_stream(stream = "custom-stream-name", max_msgs = 1000)]
/// pub struct AnotherEvent { /* ... */ }
/// impl Event for AnotherEvent {}
/// ```
#[proc_macro_attribute]
pub fn define_event_stream(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 属性引数をパース
    let args = parse_macro_input!(attr as StreamConfigArgs);
    // イベント構造体の定義をパース
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // --- STREAM_NAME 定数の生成 ---
    let stream_name_lit = match &args.stream_name_override {
        // & を追加
        Some(ref lit) => (*lit).clone(), // ref を使い、clone する
        None => {
            // 指定がなければ構造体名をケバブケースに変換
            let kebab_name = struct_name.to_string().to_kebab_case();
            LitStr::new(&kebab_name, struct_name.span())
        }
    };
    let stream_name_def = quote! {
        pub const STREAM_NAME: &'static str = #stream_name_lit;
    };

    // --- 個別の属性定数を生成 ---
    // StreamConfigArgs から StreamAttributes を作成
    let stream_attributes: StreamAttributes = args.into();

    // 各属性を個別の定数として生成
    let max_age_def = if let Some(ref lit) = stream_attributes.max_age {
        quote! { pub const STREAM_MAX_AGE: &'static str = #lit; }
    } else {
        quote! {}
    };

    let max_msgs_def = if let Some(ref lit) = stream_attributes.max_msgs {
        quote! { pub const STREAM_MAX_MSGS: u64 = #lit; }
    } else {
        quote! {}
    };

    let max_bytes_def = if let Some(ref lit) = stream_attributes.max_bytes {
        quote! { pub const STREAM_MAX_BYTES: u64 = #lit; }
    } else {
        quote! {}
    };

    let max_msg_size_def = if let Some(ref lit) = stream_attributes.max_msg_size {
        quote! { pub const STREAM_MAX_MSG_SIZE: u64 = #lit; }
    } else {
        quote! {}
    };

    let storage_def = if let Some(ref lit) = stream_attributes.storage {
        quote! { pub const STREAM_STORAGE: &'static str = #lit; }
    } else {
        quote! {}
    };

    let retention_def = if let Some(ref lit) = stream_attributes.retention {
        quote! { pub const STREAM_RETENTION: &'static str = #lit; }
    } else {
        quote! {}
    };

    let discard_def = if let Some(ref lit) = stream_attributes.discard {
        quote! { pub const STREAM_DISCARD: &'static str = #lit; }
    } else {
        quote! {}
    };

    let duplicate_window_def = if let Some(ref lit) = stream_attributes.duplicate_window {
        quote! { pub const STREAM_DUPLICATE_WINDOW: &'static str = #lit; }
    } else {
        quote! {}
    };

    let allow_rollup_def = if let Some(ref lit) = stream_attributes.allow_rollup {
        let value = lit.value;
        quote! { pub const STREAM_ALLOW_ROLLUP: bool = #value; }
    } else {
        quote! {}
    };

    let deny_delete_def = if let Some(ref lit) = stream_attributes.deny_delete {
        let value = lit.value;
        quote! { pub const STREAM_DENY_DELETE: bool = #value; }
    } else {
        quote! {}
    };

    let deny_purge_def = if let Some(ref lit) = stream_attributes.deny_purge {
        let value = lit.value;
        quote! { pub const STREAM_DENY_PURGE: bool = #value; }
    } else {
        quote! {}
    };

    let description_def = if let Some(ref lit) = stream_attributes.description {
        quote! { pub const STREAM_DESCRIPTION: &'static str = #lit; }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #input // 元の構造体定義を維持

        // 生成された定数を構造体の関連アイテムとして追加
        impl #struct_name {
            #stream_name_def
            #max_age_def
            #max_msgs_def
            #max_bytes_def
            #max_msg_size_def
            #storage_def
            #retention_def
            #discard_def
            #duplicate_window_def
            #allow_rollup_def
            #deny_delete_def
            #deny_purge_def
            #description_def
        }
    };

    TokenStream::from(expanded)
}
