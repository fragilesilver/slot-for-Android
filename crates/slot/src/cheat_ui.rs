//! The pause menu's cheat list: one full-screen face rebuilt whenever the list, the cursor or the
//! core changes. Rows are `<description> ........ ON/OFF`; A toggles, B goes back.
use slot_gfx::{Compositor, Draw, TexId, OUT_H, OUT_W};
use slot_ui::{edge, hint_face, opening, text, UndoFace};

use crate::app::Phase;
use crate::session::Session;

const TITLE_Y: u32 = 22;
const ROWS_TOP: f32 = 84.0;
const PITCH: f32 = 52.0;
const VISIBLE: usize = 6;
const EDGE: u32 = 32;
const LEGEND_Y: u32 = 427;
const INK: [u8; 3] = [0xf6, 0xf4, 0xef];
const DIM: [u8; 3] = [0x9a, 0x9a, 0xa4];
const LIT: [u8; 3] = [0x7d, 0xe0, 0x9a];
const BAD: [u8; 3] = [0xff, 0x8a, 0x80];
const PX: f32 = 28.0;

#[derive(Default)]
pub(crate) struct CheatPanel {
    texture: Option<TexId>,
    built_for: Option<Key>,
    visible: bool,
    cursor: usize,
}

#[derive(PartialEq, Eq, Clone)]
struct Key {
    version: u32,
    cursor: usize,
    supported: bool,
    note: String,
}

impl CheatPanel {
    pub fn update(&mut self, session: &Session, compositor: &mut Compositor) {
        let app = session.app();
        if matches!(app.phase(), Phase::Doze { .. }) {
            self.visible = false;
            return;
        }
        let Some((rows, cursor, supported, note)) = app.cheat_view() else {
            self.visible = false;
            self.built_for = None;
            return;
        };
        self.visible = true;
        self.cursor = cursor;
        let key = Key {
            version: app.cheats_version(),
            cursor,
            supported,
            note: note.clone(),
        };
        if self.built_for.as_ref() == Some(&key) && self.texture.is_some() {
            return;
        }
        let face = build(&rows, cursor, supported, &note);
        match self.texture {
            Some(id) => compositor.update_texture(id, face.w, face.h, &face.rgba),
            None => self.texture = Some(compositor.create_texture(face.w, face.h, &face.rgba)),
        }
        self.built_for = Some(key);
    }

    pub fn draw(&self, draws: &mut Vec<Draw>) {
        let (true, Some(tex)) = (self.visible, self.texture) else {
            return;
        };
        draws.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: OUT_W as f32,
            h: OUT_H as f32,
            colour: opening(),
        });
        let first = first_row(self.cursor);
        draws.push(Draw::Rect {
            x: 0.0,
            y: ROWS_TOP + PITCH * (self.cursor - first) as f32 + 4.0,
            w: OUT_W as f32,
            h: PITCH - 8.0,
            colour: edge(),
        });
        draws.push(Draw::Tex {
            x: 0.0,
            y: 0.0,
            w: OUT_W as f32,
            h: OUT_H as f32,
            tex,
            alpha: 1.0,
        });
    }
}

fn first_row(cursor: usize) -> usize {
    cursor.saturating_sub(VISIBLE - 1)
}

fn build(rows: &[(&str, bool)], cursor: usize, supported: bool, note: &str) -> UndoFace {
    let (w, h) = (OUT_W, OUT_H);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    line(&mut rgba, "Cheats", EDGE, TITLE_Y, INK, false);
    if !note.is_empty() {
        let ink = if note.starts_with("Master code never")
            || note.starts_with("The core took")
            || note.contains("condition")
        {
            BAD
        } else {
            LIT
        };
        line_w(&mut rgba, note, 170, TITLE_Y, ink, false, 420.0);
    }
    if !supported {
        line(
            &mut rgba,
            "Cheats need the mGBA core",
            EDGE,
            ROWS_TOP as u32 + 8,
            DIM,
            false,
        );
    } else if rows.is_empty() {
        line(
            &mut rgba,
            "No codes for this game",
            EDGE,
            ROWS_TOP as u32 + 8,
            DIM,
            false,
        );
        line(
            &mut rgba,
            "Put them in Cheats/<rom name>.txt",
            EDGE,
            ROWS_TOP as u32 + 8 + PITCH as u32,
            DIM,
            false,
        );
    }
    let first = first_row(cursor);
    for (i, (desc, on)) in rows.iter().enumerate().skip(first).take(VISIBLE) {
        let y = ROWS_TOP as u32 + PITCH as u32 * (i - first) as u32 + 8;
        let lit = i == cursor;
        line(&mut rgba, desc, EDGE, y, if lit { INK } else { DIM }, false);
        let (value, colour) = if *on { ("On", LIT) } else { ("Off", DIM) };
        line(&mut rgba, value, w - EDGE, y, colour, true);
    }
    if rows.len() > VISIBLE {
        let more = format!("{}/{}", cursor + 1, rows.len());
        line(&mut rgba, &more, w - EDGE, TITLE_Y, DIM, true);
    }
    for (x, face) in [(EDGE, hint_face("B", "Back")), (w / 2, hint_face("A", "Toggle"))] {
        blit(&mut rgba, w, &face, x, LEGEND_Y);
    }
    UndoFace { rgba, w, h }
}

/// Draw one line of text with its left edge (or right edge when `right`) at `x`.
pub(crate) fn line(dst: &mut [u8], s: &str, x: u32, y: u32, colour: [u8; 3], right: bool) {
    line_w(dst, s, x, y, colour, right, OUT_W as f32 - 2.0 * EDGE as f32 - 120.0);
}

/// `line`, shrunk to fit in `max_w` pixels.
pub(crate) fn line_w(
    dst: &mut [u8],
    s: &str,
    x: u32,
    y: u32,
    colour: [u8; 3],
    right: bool,
    max_w: f32,
) {
    let Some(font) = text::label_font() else {
        return;
    };
    let layout = text::fit(font, s, max_w, 1, PX, 14.0);
    let set = layout
        .lines
        .iter()
        .map(|l| text::line_width(font, l, layout.px, layout.tracking))
        .fold(0.0, f32::max);
    let (bw, bh) = (set.ceil() as u32 + 8, 40u32);
    let mut cell = vec![0u8; (bw * bh * 4) as usize];
    text::draw_centred(&mut cell, bw, bh, &layout, colour);
    let at = if right { x.saturating_sub(bw) } else { x };
    blit(dst, OUT_W, &UndoFace { rgba: cell, w: bw, h: bh }, at, y);
}

pub(crate) fn blit(dst: &mut [u8], dst_w: u32, src: &UndoFace, x: u32, y: u32) {
    for sy in 0..src.h {
        let dy = y + sy;
        if (dy as usize + 1) * dst_w as usize * 4 > dst.len() {
            break;
        }
        for sx in 0..src.w {
            let dx = x + sx;
            if dx >= dst_w {
                break;
            }
            let s = ((sy * src.w + sx) * 4) as usize;
            let d = ((dy * dst_w + dx) * 4) as usize;
            let a = src.rgba[s + 3] as u32;
            if a == 0 {
                continue;
            }
            for c in 0..3 {
                dst[d + c] = ((src.rgba[s + c] as u32 * a + dst[d + c] as u32 * (255 - a)) / 255) as u8;
            }
            dst[d + 3] = dst[d + 3].max(src.rgba[s + 3]);
        }
    }
}
