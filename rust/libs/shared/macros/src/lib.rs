use proc_macro::TokenStream;
#[cfg(test)]
use std::time::Duration;

mod define_streams;
mod event;
mod stream_worker;

#[proc_macro_attribute]
pub fn stream_worker(attr: TokenStream, item: TokenStream) -> TokenStream {
    stream_worker::stream_worker_impl(attr, item)
}

#[proc_macro]
pub fn define_streams(input: TokenStream) -> TokenStream {
    define_streams::define_streams_impl(input)
}

#[cfg(test)]
fn parse_duration(opt: &Option<String>) -> Option<Duration> {
    opt.as_ref().map(|s| {
        humantime::parse_duration(s)
            .unwrap_or_else(|e| panic!("invalid duration literal `{}`: {}", s, e))
    })
}

#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event::event_impl(attr, item)
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
