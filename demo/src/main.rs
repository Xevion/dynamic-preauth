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
const HOST_INFO: (&'static str, &'static str) = match option_env!("RAILWAY_PUBLIC_DOMAIN") {
    Some(domain) => ("https", domain),
    None => ("http", "localhost:5800"),
};

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

    if hash_match {
        eprintln!("Value has not been changed since build");

        // Only fail immediately if built in Railway CI
        if option_env!("RAILWAY_PUBLIC_DOMAIN").is_some() {
            return;
        }
    }

    let mut token = key_data.value.trim().parse::<u32>();

    if let Some(forced_token) = option_env!("FORCED_TOKEN") {
        token = forced_token.parse::<u32>();
    }

    match token {
        Ok(token) => {
            println!("Token: {:08X}", token);
            request(token);
        }
        Err(e) => {
            eprintln!("Token was changed, but is not a valid u32 integer: {}", e);
            eprintln!("Original Value: {}", key_data.value);
            return;
        }
    }

    println!("Hash match: {}", hash_match);
}

fn request(token: u32) {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&format!(
            "{}://{}/notify?key=0x{:08X}",
            HOST_INFO.0, HOST_INFO.1, token
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
                    println!("Request URL: {}", resp.url());
                    println!(
                        "Response body: \n{}",
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
}
