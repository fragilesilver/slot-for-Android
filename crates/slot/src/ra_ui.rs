//! The Achievements settings screen and the on-screen keyboard it types credentials with.
//! `Keyboard` is pure state (no drawing, no I/O) so it can be tested; `RaPanel` rasterizes
//! whatever the app says is showing into one full-screen face, like the cheat list does.
use slot_gfx::{Compositor, Draw, TexId, OUT_H, OUT_W};
use slot_ui::{edge, hint_face, opening, text, UndoFace};

use crate::app::Phase;
use crate::session::Session;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Field {
    Username,
    Password,
    /// The words on a cart's label (Edit Cart).
    CartText,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Shift,
    Symbols,
    Space,
    Delete,
    Done,
}

pub enum Press {
    Nothing,
    Done,
}

const LETTERS: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl_", "zxcvbnm.-@"];
const SYMBOLS: [&str; 4] = ["!@#$%^&*()", "-_=+[]{}\\|", ";:'\",.<>/?", "~`<>()+=-_"];
const KEYS_X: f32 = 60.0;
const KEYS_Y: f32 = 146.0;
const KEY_W: f32 = 60.0;
const KEY_H: f32 = 50.0;
const PITCH_Y: f32 = 58.0;
const WIDE_W: f32 = 120.0;
const MAX_LEN: usize = 64;

#[derive(Clone, Debug)]
pub struct Keyboard {
    pub field: Field,
    pub text: String,
    pub shift: bool,
    pub symbols: bool,
    pub row: usize,
    pub col: usize,
}

impl Keyboard {
    pub fn new(field: Field, text: &str) -> Self {
        Keyboard {
            field,
            text: text.to_string(),
            shift: false,
            symbols: false,
            row: 1,
            col: 0,
        }
    }

    pub fn secret(&self) -> bool {
        self.field == Field::Password
    }

    pub fn keys(&self) -> Vec<Vec<Key>> {
        let set = if self.symbols { SYMBOLS } else { LETTERS };
        let mut rows: Vec<Vec<Key>> = set
            .iter()
            .map(|r| {
                r.chars()
                    .map(|c| {
                        let c = if self.shift && !self.symbols {
                            c.to_ascii_uppercase()
                        } else {
                            c
                        };
                        Key::Char(c)
                    })
                    .collect()
            })
            .collect();
        rows.push(vec![
            Key::Shift,
            Key::Symbols,
            Key::Space,
            Key::Delete,
            Key::Done,
        ]);
        rows
    }

    /// The key's rectangle in output space.
    pub fn rect(row: usize, col: usize) -> [f32; 4] {
        let y = KEYS_Y + PITCH_Y * row as f32;
        if row < 4 {
            [KEYS_X + KEY_W * col as f32, y, KEY_W - 6.0, KEY_H]
        } else {
            [KEYS_X + WIDE_W * col as f32, y, WIDE_W - 6.0, KEY_H]
        }
    }

    pub fn key_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        for (r, row) in self.keys().iter().enumerate() {
            for c in 0..row.len() {
                let [kx, ky, kw, kh] = Keyboard::rect(r, c);
                if x >= kx && x <= kx + kw + 6.0 && y >= ky && y <= ky + kh + 4.0 {
                    return Some((r, c));
                }
            }
        }
        None
    }

    pub fn move_by(&mut self, dr: i32, dc: i32) {
        let rows = self.keys();
        self.row = (self.row as i32 + dr).rem_euclid(rows.len() as i32) as usize;
        let n = rows[self.row].len() as i32;
        self.col = (self.col as i32 + dc).rem_euclid(n) as usize;
    }

    pub fn backspace(&mut self) {
        self.text.pop();
    }

    /// Act on the key under the cursor.
    pub fn press(&mut self) -> Press {
        let rows = self.keys();
        let row = self.row.min(rows.len() - 1);
        let key = rows[row][self.col.min(rows[row].len() - 1)];
        match key {
            Key::Char(c) => {
                if self.text.chars().count() < MAX_LEN {
                    self.text.push(c);
                }
                // One capital at a time, like a phone.
                if self.shift && !self.symbols {
                    self.shift = false;
                }
            }
            Key::Space => {
                if self.field != Field::Username && self.text.chars().count() < MAX_LEN {
                    self.text.push(' ');
                }
            }
            Key::Shift => self.shift = !self.shift,
            Key::Symbols => {
                self.symbols = !self.symbols;
            }
            Key::Delete => self.backspace(),
            Key::Done => return Press::Done,
        }
        Press::Nothing
    }

    pub fn tap(&mut self, x: f32, y: f32) -> Press {
        match self.key_at(x, y) {
            Some((r, c)) => {
                self.row = r;
                self.col = c;
                self.press()
            }
            None => Press::Nothing,
        }
    }

    pub fn label(key: Key) -> String {
        match key {
            Key::Char(c) => c.to_string(),
            Key::Shift => "Shift".into(),
            Key::Symbols => "#+=".into(),
            Key::Space => "Space".into(),
            Key::Delete => "Del".into(),
            Key::Done => "OK".into(),
        }
    }
}

/// What the app says the Achievements screen shows.
#[derive(Clone, PartialEq)]
pub enum View {
    Menu {
        cursor: usize,
        enabled: bool,
        username: String,
        has_secret: bool,
    },
    Entry {
        field: Field,
        text: String,
        shift: bool,
        symbols: bool,
        row: usize,
        col: usize,
    },
}

const INK: [u8; 3] = [0xf6, 0xf4, 0xef];
const DIM: [u8; 3] = [0x9a, 0x9a, 0xa4];
const LIT: [u8; 3] = [0x7d, 0xe0, 0x9a];
const BAD: [u8; 3] = [0xff, 0x8a, 0x80];
const ROWS_TOP: f32 = 84.0;
const PITCH: f32 = 52.0;
const EDGE: u32 = 32;

pub const MENU_ROWS: usize = 4;

#[derive(Default)]
pub(crate) struct RaPanel {
    texture: Option<TexId>,
    built_for: Option<(View, String)>,
    shown: Option<View>,
}

fn status_line(session: &Session, enabled: bool, username: &str) -> (String, [u8; 3]) {
    use slot_achievements::SyncStatus::*;
    if !enabled {
        return ("Off".into(), DIM);
    }
    if username.is_empty() {
        return ("Needs an account".into(), BAD);
    }
    match session.achievement_sync_status() {
        _ if !session.achievements_active() => ("Starts next game".into(), DIM),
        Disabled => ("Off".into(), DIM),
        Syncing => ("Syncing".into(), LIT),
        Ready => ("Online".into(), LIT),
        Offline => ("Offline, will sync later".into(), DIM),
        Attention => ("Needs attention".into(), BAD),
    }
}

impl RaPanel {
    pub fn update(&mut self, session: &Session, compositor: &mut Compositor) {
        let app = session.app();
        if matches!(app.phase(), Phase::Doze { .. }) {
            self.shown = None;
            return;
        }
        let Some(view) = app.ra_view() else {
            self.shown = None;
            self.built_for = None;
            return;
        };
        let status = match &view {
            View::Menu {
                enabled, username, ..
            } => status_line(session, *enabled, username).0,
            View::Entry { .. } => String::new(),
        };
        let key = (view.clone(), status.clone());
        self.shown = Some(view.clone());
        if self.built_for.as_ref() == Some(&key) && self.texture.is_some() {
            return;
        }
        let face = build(&view, session);
        match self.texture {
            Some(id) => compositor.update_texture(id, face.w, face.h, &face.rgba),
            None => self.texture = Some(compositor.create_texture(face.w, face.h, &face.rgba)),
        }
        self.built_for = Some(key);
    }

    pub fn draw(&self, draws: &mut Vec<Draw>) {
        let (Some(view), Some(tex)) = (&self.shown, self.texture) else {
            return;
        };
        draws.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: OUT_W as f32,
            h: OUT_H as f32,
            colour: opening(),
        });
        match view {
            View::Menu { cursor, .. } => draws.push(Draw::Rect {
                x: 0.0,
                y: ROWS_TOP + PITCH * (*cursor as f32 + 1.0) + 4.0,
                w: OUT_W as f32,
                h: PITCH - 8.0,
                colour: edge(),
            }),
            View::Entry {
                row, col, shift, ..
            } => {
                let kb = Keyboard {
                    field: Field::Username,
                    text: String::new(),
                    shift: *shift,
                    symbols: false,
                    row: *row,
                    col: *col,
                };
                for (r, keys) in kb.keys().iter().enumerate() {
                    for c in 0..keys.len() {
                        let [x, y, w, h] = Keyboard::rect(r, c);
                        let lit = r == *row && c == *col;
                        draws.push(Draw::Rect {
                            x,
                            y,
                            w,
                            h,
                            colour: if lit { edge() } else { [0.18, 0.18, 0.21, 1.0] },
                        });
                    }
                }
                draws.push(Draw::Rect {
                    x: 60.0,
                    y: 78.0,
                    w: 594.0,
                    h: 46.0,
                    colour: [0.11, 0.11, 0.13, 1.0],
                });
            }
        }
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

fn build(view: &View, session: &Session) -> UndoFace {
    let (w, h) = (OUT_W, OUT_H);
    let mut px = vec![0u8; (w * h * 4) as usize];
    match view {
        View::Menu {
            enabled,
            username,
            has_secret,
            ..
        } => {
            glyphs(&mut px, "Achievements", EDGE as f32, 26.0, 30.0, INK, false);
            let (status, ink) = status_line(session, *enabled, username);
            let rows: [(&str, String, [u8; 3]); MENU_ROWS + 1] = [
                ("Status", status, ink),
                ("Enabled", if *enabled { "On" } else { "Off" }.into(), INK),
                (
                    "Username",
                    if username.is_empty() {
                        "Not set".into()
                    } else {
                        username.clone()
                    },
                    INK,
                ),
                (
                    "Password",
                    if *has_secret { "Saved" } else { "Not set" }.into(),
                    INK,
                ),
                ("Log out", String::new(), INK),
            ];
            for (i, (label, value, ink)) in rows.iter().enumerate() {
                let y = ROWS_TOP + PITCH * i as f32 + 8.0;
                let dim = i == 0;
                glyphs(&mut px, label, EDGE as f32, y, 28.0, if dim { DIM } else { INK }, false);
                if !value.is_empty() {
                    glyphs(&mut px, value, (w - EDGE) as f32, y, 28.0, *ink, true);
                }
            }
            for (x, face) in [
                (EDGE, hint_face("B", "Back")),
                (w / 2 - 40, hint_face("A", "Change")),
            ] {
                blit(&mut px, w, &face, x, 427);
            }
        }
        View::Entry {
            field,
            text,
            shift,
            symbols,
            ..
        } => {
            let title = match field {
                Field::Username => "RetroAchievements username",
                Field::Password => "RetroAchievements password",
                Field::CartText => "Words on the label",
            };
            glyphs(&mut px, title, 60.0, 30.0, 24.0, INK, false);
            let shown: String = match field {
                Field::Password => "\u{2022}".repeat(text.chars().count()),
                Field::Username | Field::CartText => text.clone(),
            };
            let tail: String = shown
                .chars()
                .rev()
                .take(34)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            glyphs(&mut px, &format!("{tail}_"), 72.0, 90.0, 26.0, INK, false);
            let kb = Keyboard {
                field: *field,
                text: String::new(),
                shift: *shift,
                symbols: *symbols,
                row: 0,
                col: 0,
            };
            for (r, keys) in kb.keys().iter().enumerate() {
                for (c, key) in keys.iter().enumerate() {
                    let [x, y, kw, kh] = Keyboard::rect(r, c);
                    let label = Keyboard::label(*key);
                    let px_size = if label.chars().count() > 1 { 20.0 } else { 26.0 };
                    let lit = match key {
                        Key::Shift => *shift,
                        Key::Symbols => *symbols,
                        _ => false,
                    };
                    let ink = if lit { LIT } else { INK };
                    let at = x + kw / 2.0 - measure(&label, px_size) / 2.0;
                    glyphs(&mut px, &label, at, y + (kh - px_size) / 2.0 - 2.0, px_size, ink, false);
                }
            }
            for (x, face) in [
                (60, hint_face("B", "Delete")),
                (230, hint_face("SELECT", "Shift")),
                (430, hint_face("START", "Done")),
            ] {
                blit(&mut px, w, &face, x, 436);
            }
        }
    }
    UndoFace {
        rgba: px,
        w,
        h,
    }
}

fn measure(s: &str, size: f32) -> f32 {
    let Some(font) = text::label_font() else {
        return 0.0;
    };
    s.chars().map(|c| font.metrics(c, size).advance_width).sum()
}

/// Draw `s` with its top near `y` (left edge at `x`, or right edge when `right`). Unlike
/// `text::fit` this keeps the case it was given.
fn glyphs(dst: &mut [u8], s: &str, x: f32, y: f32, size: f32, ink: [u8; 3], right: bool) {
    let Some(font) = text::label_font() else {
        return;
    };
    let width: f32 = s.chars().map(|c| font.metrics(c, size).advance_width).sum();
    let mut pen = if right { x - width } else { x };
    let Some(lm) = font.horizontal_line_metrics(size) else {
        return;
    };
    let baseline = y + lm.ascent;
    for c in s.chars() {
        let (m, cov) = font.rasterize(c, size);
        let gx = (pen + m.xmin as f32).round() as i32;
        let gy = (baseline - (m.height as f32 + m.ymin as f32)).round() as i32;
        for ry in 0..m.height as i32 {
            for rx in 0..m.width as i32 {
                let (dx, dy) = (gx + rx, gy + ry);
                if dx < 0 || dy < 0 || dx >= OUT_W as i32 || dy >= OUT_H as i32 {
                    continue;
                }
                let a = cov[(ry as usize) * m.width + rx as usize] as u32;
                if a == 0 {
                    continue;
                }
                let at = ((dy as u32 * OUT_W + dx as u32) * 4) as usize;
                for k in 0..3 {
                    dst[at + k] =
                        ((ink[k] as u32 * a + dst[at + k] as u32 * (255 - a)) / 255) as u8;
                }
                dst[at + 3] = dst[at + 3].max(a as u8);
            }
        }
        pen += m.advance_width;
    }
}

fn blit(dst: &mut [u8], dst_w: u32, src: &UndoFace, x: u32, y: u32) {
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
            for k in 0..3 {
                dst[d + k] = ((src.rgba[s + k] as u32 * a + dst[d + k] as u32 * (255 - a)) / 255) as u8;
            }
            dst[d + 3] = dst[d + 3].max(src.rgba[s + 3]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press_char(kb: &mut Keyboard, c: char) {
        'find: for (r, row) in kb.keys().iter().enumerate() {
            for (col, k) in row.iter().enumerate() {
                if *k == Key::Char(c) {
                    kb.row = r;
                    kb.col = col;
                    break 'find;
                }
            }
        }
        kb.press();
    }

    #[test]
    fn typing_a_name_with_one_shifted_letter() {
        let mut kb = Keyboard::new(Field::Username, "");
        kb.shift = true;
        press_char(&mut kb, 'A');
        press_char(&mut kb, 'b');
        assert_eq!(kb.text, "Ab");
        assert!(!kb.shift, "shift is for one letter");
    }

    #[test]
    fn done_is_reported_and_taps_find_keys() {
        let mut kb = Keyboard::new(Field::Password, "x");
        let [x, y, w, h] = Keyboard::rect(4, 4);
        assert!(matches!(kb.tap(x + w / 2.0, y + h / 2.0), Press::Done));
        let [x, y, w, h] = Keyboard::rect(1, 0);
        kb.tap(x + w / 2.0, y + h / 2.0);
        assert_eq!(kb.text, "xq");
        assert!(kb.key_at(5.0, 5.0).is_none());
    }

    #[test]
    fn delete_and_wrap() {
        let mut kb = Keyboard::new(Field::Username, "ab");
        kb.backspace();
        assert_eq!(kb.text, "a");
        kb.row = 0;
        kb.move_by(-1, 0);
        assert_eq!(kb.row, 4);
        kb.col = 4;
        kb.move_by(0, 1);
        assert_eq!(kb.col, 0);
    }
}
