use qrcode::QrCode;
use qrcode::render::{svg, unicode};
use std::fs;
use std::path::Path;
use std::process;

pub fn run(args: &[String]) {
    let mut content: Option<String> = None;
    let mut output: Option<String> = None;
    let mut level = qrcode::EcLevel::M;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-o/--output requires a file path");
                    process::exit(1);
                }
                output = Some(args[i].clone());
            }
            "-l" | "--level" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("-l/--level requires a level (L/M/Q/H)");
                    process::exit(1);
                }
                level = match args[i].to_uppercase().as_str() {
                    "L" => qrcode::EcLevel::L,
                    "M" => qrcode::EcLevel::M,
                    "Q" => qrcode::EcLevel::Q,
                    "H" => qrcode::EcLevel::H,
                    _ => {
                        eprintln!("Invalid error correction level: {}. Use L, M, Q, or H", args[i]);
                        process::exit(1);
                    }
                };
            }
            "-h" | "--help" => {
                print_encode_help();
                process::exit(0);
            }
            other if other.starts_with('-') => {
                eprintln!("Unknown option: {other}");
                print_encode_help();
                process::exit(1);
            }
            _ => {
                if content.is_some() {
                    eprintln!("Multiple content arguments provided");
                    process::exit(1);
                }
                content = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let content = content.unwrap_or_else(|| {
        eprintln!("No content provided");
        print_encode_help();
        process::exit(1);
    });

    let code = QrCode::with_error_correction_level(&content, level).unwrap_or_else(|e| {
        eprintln!("Error generating QR code: {e}");
        process::exit(1);
    });

    match output {
        Some(path) => save_to_file(&code, &path),
        None => print_to_terminal(&code),
    }
}

fn print_encode_help() {
    eprintln!("Usage: qrc encode <content> [options]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -o, --output <file>   Save QR code to file (PNG or SVG)");
    eprintln!("  -l, --level <level>   Error correction level: L, M, Q, H (default: M)");
    eprintln!("  -h, --help            Print this help");
}

fn print_to_terminal(code: &QrCode) {
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{image}");
}

fn save_to_file(code: &QrCode, path: &str) {
    let path = Path::new(path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    match ext.as_str() {
        "svg" => {
            let svg = code.render::<svg::Color>().build();
            fs::write(path, svg).unwrap_or_else(|e| {
                eprintln!("Error writing SVG: {e}");
                process::exit(1);
            });
        }
        _ => {
            let image = code.render::<image::Luma<u8>>().build();
            image.save(path).unwrap_or_else(|e| {
                eprintln!("Error saving image: {e}");
                process::exit(1);
            });
        }
    }

    let abs_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let abs_str = abs_path.to_string_lossy();
    // Remove \\?\ prefix on Windows
    let display = abs_str.strip_prefix(r"\\?\").unwrap_or(&abs_str);
    println!("{display}");
}
