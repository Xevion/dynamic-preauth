use std::{
    env,
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("long_string.txt");
    let mut f = BufWriter::new(File::create(&dest_path)?);

    let long_string = "abc".repeat(100);
    write!(f, "{}", long_string)?;

    Ok(())
}
