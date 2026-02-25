use crate::barcode;

/// Generate EPS content for ISBN barcode with optional EAN-5 add-on
/// All coordinates are in mm, then scaled to pt via PostScript `sc` command
pub fn generate_eps(
    isbn: &str,
    addon: &str,
    bar_height_mm: f64,
    dpi: u32,
    addon_offset_mm: f64,
) -> Option<String> {
    let ean13_modules = barcode::encode_ean13(isbn)?;
    let ean5_modules = if !addon.is_empty() {
        barcode::encode_ean5(addon)
    } else {
        None
    };

    // Module width (X-dimension) in mm
    let x_dim: f64 = 0.33;
    let scale = 2.83464567; // mm to pt
    let font_size: f64 = 3.175; // ~9pt in mm space
    // All values matched to reference file (978896993046013590.eps)
    // bar_height_mm = guard bar full height (guard_bottom to bar_top)
    // Reference: guard_bottom=1.093, bar_bottom=2.743, bar_top=16.093 (=1.093+15)
    // text_y=0.0847, addon_text_y=13.7647, addon_bar_top=13.35, addon_bar_bottom=1.093

    let text_y: f64 = 0.0847;       // ISBN text baseline (fixed)
    let guard_bottom: f64 = 1.093;   // guard bars & addon bars bottom (fixed)
    let bar_bottom: f64 = 2.743;     // normal bars bottom (fixed)
    let bar_top: f64 = guard_bottom + bar_height_mm; // bar_height = full guard height
    let quiet_zone_left: f64 = 3.63;

    // Add-on: text baseline sits 0.4147mm above addon bar top
    // addon bar top is offset from bar_top by (font_size + 0.4147) to align text top with bar_top
    let addon_gap_modules: f64 = 7.0;
    let addon_bar_bottom: f64 = guard_bottom; // fixed, same as guard
    let addon_text_baseline_gap: f64 = 0.4147; // gap between addon bar top and text baseline
    let addon_text_y: f64 = bar_top - font_size + addon_offset_mm;
    let addon_bar_top: f64 = addon_text_y - addon_text_baseline_gap;

    // Calculate total width in mm
    let ean13_width_mm = ean13_modules.len() as f64 * x_dim;
    let quiet_zone_right: f64 = 2.31;
    let addon_section_width = if ean5_modules.is_some() {
        let addon_modules_count = ean5_modules.as_ref().unwrap().len() as f64;
        addon_gap_modules * x_dim + addon_modules_count * x_dim + 2.0
    } else {
        0.0
    };
    let total_width_mm = quiet_zone_left + ean13_width_mm + quiet_zone_right + addon_section_width;
    let total_height_mm = bar_top + 0.5;

    // Convert to pt for BoundingBox
    let bb_width = (total_width_mm * scale).ceil() as i32;
    let bb_height = (total_height_mm * scale).ceil() as i32;
    let bb_width_hi = total_width_mm * scale;
    let bb_height_hi = total_height_mm * scale;

    let mut eps = String::new();

    // EPS Header
    eps.push_str("%!PS-Adobe-2.0 EPSF-1.2\n");
    eps.push_str(&format!("%%BoundingBox: 0 0 {} {}\n", bb_width, bb_height));
    eps.push_str(&format!("%%HiResBoundingBox: 0 0 {:.5} {:.5}\n", bb_width_hi, bb_height_hi));
    eps.push_str("%%Creator: ISBN Barcode Maker\n");
    eps.push_str("%%EndComments\n\n");

    // Settings as comments
    eps.push_str("%SETTINGS\n");
    eps.push_str("% Color: CMYK\n");
    eps.push_str("% Background: 0.000 0.000 0.000 0.000\n");
    eps.push_str("% Foreground: 0.000 0.000 0.000 1.000\n");
    eps.push_str("% Human Readable: Yes\n");
    eps.push_str("% Text Font: Arial\n");
    eps.push_str(&format!("% Output DPI: {}\n", dpi));
    eps.push_str("% Symbology: ISBN\n");
    eps.push_str(&format!("% Value: {}\n", isbn));
    if !addon.is_empty() {
        eps.push_str(&format!("% Add-On: {}\n", addon));
    }
    eps.push_str(&format!("% X-Dimension: {:.8} mm\n", x_dim));
    eps.push_str(&format!("% Bar Height: {:.8} mm\n", bar_height_mm));
    eps.push_str(&format!("% Add-On Offset: {:.4} mm\n", addon_offset_mm));
    eps.push('\n');

    // PostScript abbreviations
    eps.push_str("/bd {bind def} bind def\n");
    eps.push_str("/c {closepath} bd\n");
    eps.push_str("/f {fill} bd\n");
    eps.push_str("/l {lineto} bd\n");
    eps.push_str("/m {moveto} bd\n");
    eps.push_str("/n {newpath} bd\n");
    eps.push_str("/r {rotate} bd\n");
    eps.push_str("/sc {scale} bd\n");
    eps.push_str("/s {show} bd\n");
    eps.push_str("/t {translate} bd\n\n");

    // Scale mm to pt
    eps.push_str(&format!("{:.8} {:.8} sc\n", scale, scale));

    // Set CMYK black
    eps.push_str("0.000 0.000 0.000 1.000 setcmykcolor\n");

    // Font: Arial fixed
    eps.push_str(&format!("/ArialMT findfont {:.7} scalefont setfont\n", font_size));

    // Helper: draw a filled rectangle (bar)
    let draw_bar = |eps: &mut String, x: f64, w: f64, y_bot: f64, y_top: f64| {
        eps.push_str(&format!(
            "n {:.4} {:.4} m {:.4} {:.4} l {:.4} {:.4} l {:.4} {:.4} l f c\n",
            x, y_bot,
            x, y_top,
            x + w, y_top,
            x + w, y_bot
        ));
    };

    let draw_text = |eps: &mut String, x: f64, y: f64, text: &str| {
        eps.push_str(&format!("n {:.4} {:.4} m ({}) s c\n", x, y, text));
    };

    // Parse ISBN digits
    let digits: Vec<u32> = isbn.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // Draw EAN-13 bars
    let mut x = quiet_zone_left;
    let module_count = ean13_modules.len();

    // First digit (left of start guard)
    let first_digit_x = quiet_zone_left - font_size * 0.9;
    draw_text(&mut eps, first_digit_x, text_y, &digits[0].to_string());

    for (i, &module) in ean13_modules.iter().enumerate() {
        if module == 1 {
            let is_guard = i < 3 || (i >= 45 && i <= 49) || i >= (module_count - 3);
            let y_bot = if is_guard { guard_bottom } else { bar_bottom };
            draw_bar(&mut eps, x, x_dim, y_bot, bar_top);
        }
        x += x_dim;
    }

    // Left group digits (1-6)
    for i in 0..6 {
        let digit = digits[i + 1];
        let module_start = 3 + i * 7;
        let digit_center_x = quiet_zone_left + (module_start as f64 + 3.5) * x_dim;
        let text_x = digit_center_x - font_size * 0.3;
        draw_text(&mut eps, text_x, text_y, &digit.to_string());
    }

    // Right group digits (7-12)
    for i in 0..6 {
        let digit = digits[i + 7];
        let module_start = 50 + i * 7;
        let digit_center_x = quiet_zone_left + (module_start as f64 + 3.5) * x_dim;
        let text_x = digit_center_x - font_size * 0.3;
        draw_text(&mut eps, text_x, text_y, &digit.to_string());
    }

    // Draw EAN-5 add-on if present
    if let Some(ref addon_modules) = ean5_modules {
        let addon_digits: Vec<u32> = addon.chars().map(|c| c.to_digit(10).unwrap()).collect();
        let addon_x_start = quiet_zone_left + ean13_width_mm + addon_gap_modules * x_dim;
        let mut ax = addon_x_start;

        for &module in addon_modules.iter() {
            if module == 1 {
                draw_bar(&mut eps, ax, x_dim, addon_bar_bottom, addon_bar_top);
            }
            ax += x_dim;
        }

        // Add-on digit text above bars
        for i in 0..5 {
            let module_offset = 4.0 + i as f64 * 9.0 + 3.5;
            let text_x = addon_x_start + module_offset * x_dim - font_size * 0.3;
            draw_text(&mut eps, text_x, addon_text_y, &addon_digits[i].to_string());
        }
    }

    eps.push_str("showpage\n");

    Some(eps)
}
