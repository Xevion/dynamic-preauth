use serde::Serialize;
use sha2::Digest;
use std::{
    env,
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

// Shared between build.rs and main.rs
#[derive(Serialize, Debug)]
struct KeyData<'a> {
    value: &'a str,
    // value_hash is not intended to be a secure trusted hash; I don't know if there's a way to ensure it stays unmodified regardless
    value_hash: String,
    compile_time: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("key.json");
    let mut f = BufWriter::new(File::create(&dest_path)?);

    let compile_time = chrono::Utc::now().to_rfc3339();
    let value = "Hello, world!";
    let value_hash = sha2::Sha256::digest(value.as_bytes());

    let key_data = KeyData {
        value,
        value_hash: hex::encode(value_hash),
        compile_time,
    };

    let json_data = serde_json::to_string(&key_data)?;
    write!(f, "{}", json_data.to_string())?;

    Ok(())
}
