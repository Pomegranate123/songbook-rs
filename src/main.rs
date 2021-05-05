use std::process;

fn main() {
    if let Err(e) = gpro::run() {
        println!("Error: {}", e);
        process::exit(1);
    }
}
