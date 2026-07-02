use eframe::egui;
use image::DynamicImage;

use crate::decode;

enum State {
    Selecting,
    /// Single result: print and close on next frame
    Done(String),
    /// Multiple results: show clickable labels
    Choosing(Vec<decode::QrResult>),
    Close,
}

struct ScreencapApp {
    texture: egui::TextureHandle,
    screenshot: DynamicImage,
    sel_start: Option<egui::Pos2>,
    sel_end: Option<egui::Pos2>,
    state: State,
}

impl ScreencapApp {
    fn new(cc: &eframe::CreationContext<'_>, screenshot: DynamicImage) -> Self {
        let rgba = screenshot.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_raw();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
        let texture = cc
            .egui_ctx
            .load_texture("screenshot", color_image, egui::TextureOptions::LINEAR);
        Self {
            texture,
            screenshot,
            sel_start: None,
            sel_end: None,
            state: State::Selecting,
        }
    }

    fn decode_selection(&mut self, panel_rect: egui::Rect) {
        if let (Some(start), Some(end)) = (self.sel_start, self.sel_end) {
            let sel_rect = egui::Rect::from_two_pos(start, end);
            let w = sel_rect.width().abs();
            let h = sel_rect.height().abs();
            if w <= 5.0 || h <= 5.0 {
                return;
            }
            let img_w = self.screenshot.width() as f32;
            let img_h = self.screenshot.height() as f32;
            let sx = img_w / panel_rect.width().max(1.0);
            let sy = img_h / panel_rect.height().max(1.0);

            let x1 = (sel_rect.min.x * sx).max(0.0) as u32;
            let y1 = (sel_rect.min.y * sy).max(0.0) as u32;
            let x2 = (sel_rect.max.x * sx).min(img_w) as u32;
            let y2 = (sel_rect.max.y * sy).min(img_h) as u32;

            let crop_w = x2.saturating_sub(x1).max(1);
            let crop_h = y2.saturating_sub(y1).max(1);

            let cropped = self.screenshot.crop_imm(x1, y1, crop_w, crop_h);
            let mut results = decode::decode_qr(&cropped);

            if results.is_empty() {
                eprintln!("No QR code found in selected region");
                self.sel_start = None;
                self.sel_end = None;
            } else if results.len() == 1 {
                self.state = State::Done(results.remove(0).content);
            } else {
                // Multiple: adjust centers from cropped coords to full image coords
                let adjusted: Vec<decode::QrResult> = results
                    .into_iter()
                    .map(|r| decode::QrResult {
                        content: r.content,
                        center: (r.center.0 + x1 as f32, r.center.1 + y1 as f32),
                    })
                    .collect();
                self.state = State::Choosing(adjusted);
            }
        }
    }

}

impl eframe::App for ScreencapApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.state = State::Close;
            return;
        }

        let panel_rect = ui.ctx().viewport_rect();

        // --- Background: screenshot + dark overlay ---
        let painter = ui.painter_at(panel_rect);
        painter.image(
            self.texture.id(),
            panel_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        painter.rect_filled(
            panel_rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
        );

        // Draw existing selection rectangle
        if let (Some(start), Some(end)) = (self.sel_start, self.sel_end) {
            let sel_rect = egui::Rect::from_two_pos(start, end);
            let uv_min = egui::pos2(
                (sel_rect.min.x / panel_rect.width().max(1.0)).clamp(0.0, 1.0),
                (sel_rect.min.y / panel_rect.height().max(1.0)).clamp(0.0, 1.0),
            );
            let uv_max = egui::pos2(
                (sel_rect.max.x / panel_rect.width().max(1.0)).clamp(0.0, 1.0),
                (sel_rect.max.y / panel_rect.height().max(1.0)).clamp(0.0, 1.0),
            );
            painter.image(
                self.texture.id(),
                sel_rect,
                egui::Rect::from_min_max(uv_min, uv_max),
                egui::Color32::WHITE,
            );
            painter.rect_stroke(
                sel_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 174, 255)),
                egui::StrokeKind::Inside,
            );
        }
        drop(painter);

        // --- State logic ---
        match self.state {
            State::Selecting => {
                let response = ui.interact(
                    panel_rect,
                    egui::Id::new("screenshot_overlay"),
                    egui::Sense::click_and_drag(),
                );

                if response.drag_started() {
                    self.sel_start = response.interact_pointer_pos();
                    self.sel_end = self.sel_start;
                }

                if response.is_pointer_button_down_on() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.sel_end = Some(pos);
                    }
                }

                if response.drag_stopped() {
                    self.decode_selection(panel_rect);
                }

                if self.sel_start.is_none() {
                    ui.painter_at(panel_rect).text(
                        panel_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Drag to select QR code region. Press Esc to cancel.",
                        egui::FontId::proportional(24.0),
                        egui::Color32::WHITE,
                    );
                }
            }
            State::Done(ref content) => {
                println!("{}",content);
                self.state = State::Close;
                return;
            }
            State::Choosing(ref results) => {
                let results: Vec<_> = results.iter().collect();

                // Multiple results: show clickable label on each QR code
                let img_w = self.screenshot.width() as f32;
                let img_h = self.screenshot.height() as f32;
                let sx = panel_rect.width().max(1.0) / img_w;
                let sy = panel_rect.height().max(1.0) / img_h;

                for (i, qr) in results.iter().enumerate() {
                    // Map QR center from image coords to screen coords
                    let screen_x = panel_rect.min.x + qr.center.0 * sx;
                    let screen_y = panel_rect.min.y + qr.center.1 * sy;
                    let center = egui::pos2(screen_x, screen_y);

                    let label_text = format!("[{}] Click to select", i + 1);
                    let font_id = egui::FontId::proportional(16.0);
                    let painter = ui.painter_at(panel_rect);

                    // Measure text for background rect
                    let galley = painter.layout_no_wrap(
                        label_text.clone(),
                        font_id.clone(),
                        egui::Color32::WHITE,
                    );
                    let text_size = galley.size();
                    let padding = egui::vec2(16.0, 8.0);
                    let rect_size = text_size + padding * 2.0;
                    let label_rect =
                        egui::Rect::from_center_size(center, rect_size);

                    // Draw pill background
                    let bg_color = if ui.rect_contains_pointer(label_rect) {
                        egui::Color32::from_rgba_unmultiplied(0, 120, 255, 220)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(40, 40, 40, 200)
                    };
                    painter.rect_filled(label_rect, rect_size.y / 2.0, bg_color);
                    painter.rect_stroke(
                        label_rect,
                        rect_size.y / 2.0,
                        egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 174, 255)),
                        egui::StrokeKind::Outside,
                    );

                    // Draw text
                    painter.galley(
                        egui::pos2(
                            center.x - text_size.x / 2.0,
                            center.y - text_size.y / 2.0,
                        ),
                        galley,
                        egui::Color32::WHITE,
                    );

                    // Click detection
                    let id = egui::Id::new("qr_label").with(i);
                    let resp = ui.interact(label_rect, id, egui::Sense::click());
                    if resp.clicked() {
                        println!("{}", qr.content);
                        self.state = State::Close;
                        return;
                    }
                }
            }
            State::Close => {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }

        ui.ctx().request_repaint();
    }
}

fn parse_args(args: &[String]) -> (bool, Option<usize>) {
    let mut list = false;
    let mut screen: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--list" => list = true,
            "--screen" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--screen requires a monitor number");
                    std::process::exit(1);
                }
                screen = Some(args[i].parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("Invalid screen number: {}", args[i]);
                    std::process::exit(1);
                }));
            }
            other => {
                eprintln!("Unknown argument: {other}");
                eprintln!("Usage: qrc screenshot [--list] [--screen <n>]");
                std::process::exit(1);
            }
        }
        i += 1;
    }
    (list, screen)
}

pub fn run(args: &[String]) {
    let (list_mode, screen_num) = parse_args(args);

    let monitors = xcap::Monitor::all().unwrap_or_else(|e| {
        eprintln!("Error getting monitors: {e}");
        std::process::exit(1);
    });

    if monitors.is_empty() {
        eprintln!("No monitors found");
        std::process::exit(1);
    }

    if list_mode {
        for (i, m) in monitors.iter().enumerate() {
            let primary = if m.is_primary().unwrap_or(false) { " (primary)" } else { "" };
            let name = m.name().unwrap_or_else(|_| "<unknown>".to_string());
            let w = m.width().unwrap_or(0);
            let h = m.height().unwrap_or(0);
            let x = m.x().unwrap_or(0);
            let y = m.y().unwrap_or(0);
            println!("  {}{}: {} - {}x{}+{},{}", i + 1, primary, name, w, h, x, y);
        }
        std::process::exit(0);
    }

    let monitor = if let Some(num) = screen_num {
        if num == 0 || num > monitors.len() {
            eprintln!(
                "Invalid screen number: {num}. Valid range: 1-{}. Use --list to see available screens.",
                monitors.len()
            );
            std::process::exit(1);
        }
        &monitors[num - 1]
    } else {
        monitors
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .unwrap_or_else(|| {
                eprintln!("No primary monitor found");
                std::process::exit(1);
            })
    };

    let xcap_image = monitor.capture_image().unwrap_or_else(|e| {
        eprintln!("Error capturing screen: {e}");
        std::process::exit(1);
    });

    let (w, h) = (xcap_image.width(), xcap_image.height());
    let raw = xcap_image.into_raw();
    let screenshot = DynamicImage::ImageRgba8(image::ImageBuffer::from_raw(w, h, raw).unwrap());

    let mon_x = monitor.x().unwrap_or(0) as f32;
    let mon_y = monitor.y().unwrap_or(0) as f32;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_position(egui::pos2(mon_x, mon_y))
            .with_fullscreen(true)
            .with_decorations(false)
            .with_window_level(egui::WindowLevel::AlwaysOnTop),
        ..Default::default()
    };

    eframe::run_native(
        "qrc screenshot",
        native_options,
        Box::new(move |cc| Ok(Box::new(ScreencapApp::new(cc, screenshot)))),
    )
    .unwrap_or_else(|e| {
        eprintln!("Error running overlay: {e}");
        std::process::exit(1);
    });
}
