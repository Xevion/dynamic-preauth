static LONG_STRING: &'static str = include_str!(concat!(env!("OUT_DIR"), "/long_string.txt"));

fn main() {
    println!("This package was compiled at {}", LONG_STRING);
}
