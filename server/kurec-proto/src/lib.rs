mod kurec_proto {
    include!(concat!(env!("OUT_DIR"), "/kurec.rs"));
}

pub use kurec_proto::*;
pub use prost::Message;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
        let msg = kurec_proto::MirakcEventMessage {
            tuner_url: "http://tuner:40772".to_string(),
            event: "event".to_string(),
            data: "data".to_string(),
        };
    }
}
