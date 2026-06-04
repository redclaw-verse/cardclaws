//! Renders the pass strip image and other bundled PNGs.
//!
//! Phase 1 renders a solid fill derived from the card's background color, at the
//! PassKit strip dimensions (1125x432 @3x, PRD §6.3.4). A later milestone will
//! composite the actual card-face top-third; the bundle contract (a real PNG
//! hashed into the manifest) is identical either way.

use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};

use crate::error::WalletError;

/// PassKit strip image size at @3x (PRD §6.3.4).
pub const STRIP_WIDTH: u32 = 1125;
pub const STRIP_HEIGHT: u32 = 432;

/// Render the strip image as a solid fill of `background_hex`.
pub fn render_strip(background_hex: &str) -> Result<Vec<u8>, WalletError> {
    render_solid(STRIP_WIDTH, STRIP_HEIGHT, background_hex)
}

/// Render a solid-color PNG. Used for the strip and for placeholder brand glyphs
/// until real assets are supplied.
pub fn render_solid(width: u32, height: u32, hex: &str) -> Result<Vec<u8>, WalletError> {
    let [r, g, b] = parse_hex(hex)?;
    let img = RgbaImage::from_pixel(width, height, Rgba([r, g, b, 255]));
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| WalletError::Render(e.to_string()))?;
    Ok(buf.into_inner())
}

/// Parse `#RRGGBB` (or `RRGGBB`) into RGB bytes.
fn parse_hex(hex: &str) -> Result<[u8; 3], WalletError> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return Err(WalletError::Render(format!("invalid hex color: {hex}")));
    }
    let parse = |s: &str| u8::from_str_radix(s, 16);
    match (parse(&h[0..2]), parse(&h[2..4]), parse(&h[4..6])) {
        (Ok(r), Ok(g), Ok(b)) => Ok([r, g, b]),
        _ => Err(WalletError::Render(format!("invalid hex color: {hex}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_is_valid_png_of_expected_size() {
        let bytes = render_strip("#101014").unwrap();
        // PNG magic number.
        assert_eq!(
            &bytes[0..8],
            &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]
        );
        let decoded = image::load_from_memory(&bytes).unwrap();
        assert_eq!(decoded.width(), STRIP_WIDTH);
        assert_eq!(decoded.height(), STRIP_HEIGHT);
    }

    #[test]
    fn rejects_bad_hex() {
        assert!(render_solid(10, 10, "nope").is_err());
        assert!(render_solid(10, 10, "#12345").is_err());
    }
}
