use serde::{Deserialize, Serialize};
use sha2::Digest;

// Shared between build.rs and main.rs
#[derive(Serialize, Deserialize, Debug)]
struct KeyData<'a> {
    value: &'a str,
    value_hash: String,
    compile_time: String,
}

static KEY: &'static str = include_str!(concat!(env!("OUT_DIR"), "/key.json"));

fn main() {
    let key_data: KeyData = serde_json::from_str(KEY).unwrap();

    // Print the key data
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--help".to_string()) {
        println!("Usage: {} [--json] [--help]", args[0]);
        println!("--json: Print the key data as JSON");
        println!("--help: Print this help message");
        return;
    } else if args.contains(&"--json".to_string()) {
        println!("{}", serde_json::to_string_pretty(&key_data).unwrap());
        return;
    }

    // Check the hash of the value
    let value_hash = sha2::Sha256::digest(key_data.value.as_bytes());
    let hash_match = hex::encode(value_hash) == key_data.value_hash;

    println!("Hash match: {}", hash_match);
}
