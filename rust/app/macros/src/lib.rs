//! アプリケーション層のマクロを提供するクレート
//!
//! このクレートは、アプリケーション層で使用するマクロを提供します。
//! 特に、イベント型とJetStreamストリームの設定情報を関連付けるためのマクロを提供します。

use heck::ToKebabCase;
use humantime::parse_duration;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};

mod config_parser;
use config_parser::{StreamAttributes, StreamConfigArgs};

/// イベント型に対応するEventStreamのインスタンスを生成するマクロ。
///
/// このマクロは、対象の構造体に以下の関連定数を実装します:
/// - `EVENT_STREAM: EventStream<Self>`: イベント型に対応するEventStreamのインスタンス。
///   - ストリーム名: デフォルトは構造体名をケバブケース (`kebab-case`) に変換したもの。
///   - 属性 `stream = "..."` で上書き可能。
///   - その他の属性 (`max_age`, `storage` など) で設定を指定可能。
///
/// # 使用例
/// ```ignore
/// use app_macros::define_event_stream;
/// use domain::event::Event;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// #[define_event_stream(max_age = "14d", storage = "file")]
/// pub struct MyEvent { /* ... */ }
/// impl Event for MyEvent {}
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

    // --- STREAM_NAME の生成 ---
    let stream_name_lit = match &args.stream_name_override {
        Some(ref lit) => (*lit).clone(),
        None => {
            // 指定がなければ構造体名をケバブケースに変換
            let kebab_name = struct_name.to_string().to_kebab_case();
            LitStr::new(&kebab_name, struct_name.span())
        }
    };

    // StreamConfigArgs から StreamConfig を作成
    let stream_config = StreamAttributes::from(args);

    // EventStream インスタンスの生成
    let event_stream_def = quote! {
        pub const EVENT_STREAM: ::infra_jetstream::EventStream<Self> =
            ::infra_jetstream::EventStream::new(
                #stream_name_lit,
                #stream_config
            );
    };

    let expanded = quote! {
        #input // 元の構造体定義を維持

        // EventStream インスタンスを構造体の関連アイテムとして追加
        impl #struct_name {
            #event_stream_def
        }
    };

    TokenStream::from(expanded)
}
