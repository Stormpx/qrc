use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: qrc <url_or_file_path>");
        process::exit(1);
    }

    let input = &args[1];
    let img = load_image(input).unwrap_or_else(|e| {
        eprintln!("Error loading image: {e}");
        process::exit(1);
    });

    let gray = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare_from_greyscale(gray.width() as usize, gray.height() as usize, |x, y| gray.get_pixel(x as u32, y as u32).0[0]);

    let grids = prepared.detect_grids();
    if grids.is_empty() {
        eprintln!("No QR code found in image");
        process::exit(1);
    }

    for grid in grids {
        match grid.decode() {
            Ok((_meta, content)) => println!("{content}"),
            Err(e) => eprintln!("Error decoding QR code: {e}"),
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
