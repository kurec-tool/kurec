use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::time::Duration;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitStr, Result, Token,
};

// ストリーム定義の構文解析用構造体
pub struct StreamDefinition {
    pub name: String,
    pub max_age: Option<String>,
    pub max_deliver: Option<u32>,
    pub ack_wait: Option<String>,
}

// 複数のストリーム定義を含む構造体
pub struct StreamDefinitions {
    pub streams: Vec<StreamDefinition>,
}

// 単一のストリーム定義の構文解析
impl Parse for StreamDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        // 'stream' キーワードを期待
        input.parse::<Ident>()?;

        // ストリーム名を取得
        let name = input.parse::<Ident>()?.to_string();

        // 波括弧内の設定を解析
        let content;
        braced!(content in input);

        let mut max_age = None;
        let mut max_deliver = None;
        let mut ack_wait = None;

        // 波括弧内の各設定を解析
        while !content.is_empty() {
            let key = content.parse::<Ident>()?;
            content.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "max_age" => {
                    let value = content.parse::<LitStr>()?;
                    max_age = Some(value.value());
                }
                "max_deliver" => {
                    let value = content.parse::<syn::LitInt>()?;
                    max_deliver = Some(value.base10_parse()?);
                }
                "ack_wait" => {
                    let value = content.parse::<LitStr>()?;
                    ack_wait = Some(value.value());
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "Unknown stream configuration option",
                    ))
                }
            }

            // カンマがあれば読み飛ばす
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(StreamDefinition {
            name,
            max_age,
            max_deliver,
            ack_wait,
        })
    }
}

// 複数のストリーム定義の構文解析
impl Parse for StreamDefinitions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut streams = Vec::new();

        while !input.is_empty() {
            streams.push(input.parse::<StreamDefinition>()?);
        }

        Ok(StreamDefinitions { streams })
    }
}

// Duration文字列をパース
fn parse_duration(s: &str) -> Duration {
    humantime::parse_duration(s)
        .unwrap_or_else(|e| panic!("Invalid duration literal `{}`: {}", s, e))
}

// define_streamsマクロの実装
pub fn define_streams_impl(input: TokenStream) -> TokenStream {
    // ストリーム定義を解析
    let stream_defs = parse_macro_input!(input as StreamDefinitions);

    let mut registration_code = Vec::new();

    for stream_def in &stream_defs.streams {
        let name = &stream_def.name;
        let name_str = name.to_string();

        // max_ageの処理
        let max_age = stream_def
            .max_age
            .as_ref()
            .map(|d| {
                let duration = parse_duration(d);
                let secs = duration.as_secs();
                let nanos = duration.subsec_nanos();
                quote! { Some(std::time::Duration::new(#secs, #nanos)) }
            })
            .unwrap_or(quote! { None });

        // max_deliverの処理
        let max_deliver = stream_def
            .max_deliver
            .map(|n| {
                quote! { Some(#n) }
            })
            .unwrap_or(quote! { None });

        // ack_waitの処理
        let ack_wait = stream_def
            .ack_wait
            .as_ref()
            .map(|d| {
                let duration = parse_duration(d);
                let secs = duration.as_secs();
                let nanos = duration.subsec_nanos();
                quote! { Some(std::time::Duration::new(#secs, #nanos)) }
            })
            .unwrap_or(quote! { None });

        // ストリーム登録コードを生成
        registration_code.push(quote! {
            shared_core::streams::register_stream(
                #name_str,
                shared_core::streams::StreamConfig {
                    name: #name_str.to_string(),
                    max_age: #max_age,
                    max_deliver: #max_deliver,
                    ack_wait: #ack_wait,
                }
            );
        });
    }

    // 登録コードを生成
    let expanded = quote! {
        #[ctor::ctor]
        fn register_streams() {
            #(#registration_code)*
        }
    };

    TokenStream::from(expanded)
}
