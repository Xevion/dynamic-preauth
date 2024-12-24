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

    // if hash_match {
    //     return;
    // }

    // TODO: Use token to make request
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&format!(
            "http://localhost:5800/notify?key={}",
            key_data.value
        ))
        .send();

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("Request successful");
            } else {
                println!("Request failed with status: {}", resp.status());

                if resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .map(|v| v == "application/json")
                    .unwrap_or(false)
                {
                    match resp.json::<serde_json::Value>() {
                        Ok(json_body) => {
                            println!(
                                "Response JSON: {}",
                                serde_json::to_string_pretty(&json_body).unwrap()
                            );
                        }
                        Err(e) => {
                            println!("Failed to parse JSON response: {}", e);
                        }
                    }
                } else {
                    println!(
                        "Response body: {}",
                        resp.text()
                            .unwrap_or_else(|_| "Failed to read response body".to_string())
                    );
                }
            }
        }
        Err(e) => {
            println!("Request error: {}", e);
        }
    }

    println!("Hash match: {}", hash_match);
}
