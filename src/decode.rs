use image::DynamicImage;

pub struct QrResult {
    pub content: String,
    /// Center of the QR code in image pixel coordinates (x, y)
    pub center: (f32, f32),
}

pub fn decode_qr(img: &DynamicImage) -> Vec<QrResult> {
    let gray = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare_from_greyscale(
        gray.width() as usize,
        gray.height() as usize,
        |x, y| gray.get_pixel(x as u32, y as u32).0[0],
    );

    let grids = prepared.detect_grids();
    let mut results = Vec::new();
    for grid in grids {
        if let Ok((_meta, content)) = grid.decode() {
            let [tl, tr, br, bl] = grid.bounds;
            let cx = (tl.x + tr.x + br.x + bl.x) as f32 / 4.0;
            let cy = (tl.y + tr.y + br.y + bl.y) as f32 / 4.0;
            results.push(QrResult {
                content,
                center: (cx, cy),
            });
        }
    }
    results
}
