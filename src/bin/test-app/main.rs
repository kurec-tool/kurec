// use kurec::kurec_config::get_config;

// fn main() {
//     let config = get_config("./kurec.yml").unwrap();
//     println!("Hello, world! test-app config: {:?}", config);
// }

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct S {
    #[serde(alias = "some_name")]
    some_name: i32,
}

fn main() {
    let json_str = r#"{"some_name": 1}"#;
    let s: S = serde_json::from_str(json_str).unwrap();
    println!("{:?}", s);
    println!("{}", serde_json::to_string(&s).unwrap());

    let json_str = r#"{"someName": 1}"#;
    let s: S = serde_json::from_str(json_str).unwrap();
    println!("{:?}", s);
    println!("{}", serde_json::to_string(&s).unwrap());
}
