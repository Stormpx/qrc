# qrc

A command-line tool for encoding and decoding QR codes.

## Features

- Encode content into QR codes (terminal or file output)
- Decode QR codes from local image files
- Decode QR codes from URLs (http/https)
- Interactive screen capture with region selection
- Multi-monitor support with screen selection

## Installation

```bash
cargo install --path .
```

## Usage

### Encode to QR code

```bash
# Print to terminal
qrc encode "Hello World"

# Save to PNG file
qrc encode "https://example.com" -o qrcode.png

# Save to SVG file
qrc encode "https://example.com" -o qrcode.svg

# Specify error correction level (L/M/Q/H)
qrc encode "Hello World" -o qrcode.png -l H
```

### Decode from image file

```bash
qrc <image_path>
```

### Decode from URL

```bash
qrc https://example.com/qrcode.png
```

### Screen capture mode

```bash
qrc screenshot [--list] [--screen <n>]
```

Options:
- `--list` - List all available monitors
- `--screen <n>` - Capture specific monitor (1-based index, default: primary monitor)

Examples:

```bash
# List available monitors
qrc screenshot --list

# Capture primary monitor (default)
qrc screenshot

# Capture second monitor
qrc screenshot --screen 2
```

In screen capture mode:
1. Drag to select a region containing a QR code
2. If multiple QR codes are found, click on the one you want to decode
3. Press `Esc` to cancel

### Help

```bash
qrc -h, --help
```

## Supported image formats

PNG, JPEG, GIF, BMP, WebP, and other common image formats supported by the `image` crate.

## Dependencies

- [qrcode](https://crates.io/crates/qrcode) - QR code encoding
- [rqrr](https://crates.io/crates/rqrr) - QR code decoding
- [xcap](https://crates.io/crates/xcap) - Screen capture
- [eframe](https://crates.io/crates/eframe) - GUI framework for screen selection
- [image](https://crates.io/crates/image) - Image processing
- [reqwest](https://crates.io/crates/reqwest) - HTTP client for URL downloads

## License

MIT
