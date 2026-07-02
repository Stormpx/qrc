mod decode;
mod screenshot;

use std::process;

fn print_help() {
    println!("Usage: qrc <command> [options]");
    println!();
    println!("Commands:");
    println!("  <url_or_file_path>    Decode QR code from image file or URL");
    println!("  screenshot            Select and decode QR code from screen");
    println!();
    println!("Screenshot options:");
    println!("  --list                List all available monitors");
    println!("  --screen <n>          Capture specific monitor (1-based index)");
    println!();
    println!("Other options:");
    println!("  -h, --help            Print this help message");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_help();
        process::exit(0);
    }

    match args[1].as_str() {
        "screenshot" => screenshot::run(&args[2..]),
        input => {
            let img = load_image(input).unwrap_or_else(|e| {
                eprintln!("Error loading image: {e}");
                process::exit(1);
            });

            let results = decode::decode_qr(&img);
            if results.is_empty() {
                eprintln!("No QR code found in image");
                process::exit(1);
            }
            for r in &results {
                println!("{}", r.content);
            }
        }
    }
}

fn load_image(input: &str) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    if input.starts_with("http://") || input.starts_with("https://") {
        let bytes = reqwest::blocking::get(input)?.bytes()?;
        Ok(image::load_from_memory(&bytes)?)
    } else {
        Ok(image::open(input)?)
    }
}
