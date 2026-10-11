//! The letter index above the shelf: `#` then A to Z, one slot each. Pure layout and hit-testing
//! (in the 720x480 output space) so the touch layer and the drawing agree on where a letter is.
use slot_gfx::{OUT_H, OUT_W};

pub const LETTERS: [char; 27] = [
    '#', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
    'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub const SLOT_W: f32 = 22.0;
pub const BAR_Y: f32 = 92.0;
pub const BAR_H: f32 = 30.0;
const GLYPH_PX: f32 = 17.0;

pub fn bar_x() -> f32 {
    (OUT_W as f32 - SLOT_W * LETTERS.len() as f32) / 2.0
}

/// Left edge of the nth letter's slot.
pub fn slot_x(n: usize) -> f32 {
    bar_x() + SLOT_W * n as f32
}

/// The letter whose slot holds the output-space point, with a little slack above and below so a
/// thumb does not have to be exact.
pub fn letter_at(x: f32, y: f32) -> Option<char> {
    if !(BAR_Y - 14.0..=BAR_Y + BAR_H + 14.0).contains(&y) {
        return None;
    }
    let n = ((x - bar_x()) / SLOT_W).floor();
    if n < 0.0 || n >= LETTERS.len() as f32 {
        return None;
    }
    Some(LETTERS[n as usize])
}

/// A window pixel mapped through the very rectangle the picture is drawn in (the same one the
/// compositor blits to: whole-number scale where it fits, the bezel's screen when framed).
pub fn to_output(window: (u32, u32), x: f32, y: f32) -> Option<(f32, f32)> {
    let (bx, by, bw, bh) = slot_gfx::blit_rect(window, 0.0);
    if bw <= 0 || bh <= 0 {
        return None;
    }
    // The blit rectangle counts rows from the bottom, as GL does.
    let top = window.1 as i32 - by - bh;
    Some((
        (x - bx as f32) * OUT_W as f32 / bw as f32,
        (y - top as f32) * OUT_H as f32 / bh as f32,
    ))
}

pub fn letter_at_window(window: (u32, u32), x: f32, y: f32) -> Option<char> {
    let (ox, oy) = to_output(window, x, y)?;
    letter_at(ox, oy)
}

/// One white glyph per letter, `SLOT_W` x `BAR_H`.
pub fn faces() -> Vec<slot_ui::CartFace> {
    use slot_ui::text;
    LETTERS
        .iter()
        .map(|c| {
            let (w, h) = (SLOT_W as u32, BAR_H as u32);
            let mut rgba = vec![0u8; (w * h * 4) as usize];
            if let Some(font) = text::label_font() {
                let layout = text::fit(font, &c.to_string(), SLOT_W, 1, GLYPH_PX, GLYPH_PX);
                text::draw_centred(&mut rgba, w, h, &layout, [0xf6, 0xf4, 0xef]);
            }
            slot_ui::CartFace { rgba, w, h }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_cover_hash_to_z_and_nothing_else() {
        let mid = |n: usize| slot_x(n) + SLOT_W / 2.0;
        let y = BAR_Y + BAR_H / 2.0;
        assert_eq!(letter_at(mid(0), y), Some('#'));
        assert_eq!(letter_at(mid(1), y), Some('A'));
        assert_eq!(letter_at(mid(26), y), Some('Z'));
        assert_eq!(letter_at(bar_x() - 1.0, y), None);
        assert_eq!(letter_at(slot_x(27) + 1.0, y), None);
        assert_eq!(letter_at(mid(5), 10.0), None);
    }

    #[test]
    fn a_window_pixel_goes_through_the_aspect_fit() {
        let window = (1440, 960); // exactly 2x the output
        let (x, y) = (slot_x(1) * 2.0 + SLOT_W, (BAR_Y + BAR_H / 2.0) * 2.0);
        assert_eq!(letter_at_window(window, x, y), Some('A'));
    }
}
