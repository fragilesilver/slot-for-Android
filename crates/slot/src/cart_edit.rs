//! The cart editor: pick a shell colour, finish and outline for the cart under the caret and see
//! it drawn before keeping it. Choices land in `Labels/cart_shell.txt`, the file Cart Studio
//! writes, so a card edited by hand or on the web and one edited here stay the same thing.
use slot_gfx::{Compositor, Draw, TexId, OUT_H, OUT_W};
use slot_store::{Cart, Outline, Platform, ShellChoice, ShellFinish};
use slot_ui::{cart_face_with, edge, hint_face, opening, shell_for, shell_presets, text, Finish};
use slot_ui::{Shell, UndoFace};

use crate::app::Phase;
use crate::cheat_ui::{blit, line, line_w};
use crate::label_cache::label_art;
use crate::session::Session;

pub(crate) const ROWS_TOP: f32 = 84.0;
pub(crate) const PITCH: f32 = 38.0;
pub(crate) const COLUMN_W: f32 = 392.0;
pub(crate) const LEGEND_Y: u32 = 427;
const TITLE_Y: u32 = 22;
const EDGE: u32 = 32;
const INK: [u8; 3] = [0xf6, 0xf4, 0xef];
const DIM: [u8; 3] = [0x9a, 0x9a, 0xa4];
const PREVIEW: (f32, f32, f32, f32) = (416.0, 96.0, 272.0, 240.0);

/// Where a picture comes from.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum PictureSource {
    Gallery,
    Downloads,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum EditRow {
    Preset,
    Hue,
    Vividness,
    Shade,
    Finish,
    Outline,
    Picture,
    Text,
    Reset,
}

impl EditRow {
    fn label(self) -> &'static str {
        match self {
            EditRow::Preset => "Preset",
            EditRow::Hue => "Hue",
            EditRow::Vividness => "Vividness",
            EditRow::Shade => "Shade",
            EditRow::Finish => "Finish",
            EditRow::Outline => "Outline",
            EditRow::Picture => "Picture",
            EditRow::Text => "Text",
            EditRow::Reset => "Back to default",
        }
    }
}

pub(crate) struct CartEdit {
    pub stem: String,
    cart: Cart,
    start: Option<ShellChoice>,
    default: (Shell, Outline),
    presets: Vec<(&'static str, Shell)>,
    pub cursor: usize,
    hue: f32,
    sat: f32,
    light: f32,
    finish: ShellFinish,
    outline: Outline,
    use_default: bool,
    picture_wanted: bool,
    source: PictureSource,
    pub picking: Option<std::path::PathBuf>,
    restore_wanted: bool,
    text_wanted: bool,
    text_note: &'static str,
    picture_note: &'static str,
    pub picture_changed: bool,
    pub version: u32,
}

impl CartEdit {
    pub fn open(cart: &Cart) -> CartEdit {
        let mut plain = cart.clone();
        plain.shell = None;
        let built_in = shell_for(&plain);
        let (colour, finish, outline) = match cart.shell {
            Some(c) => (c.colour, c.finish, c.outline),
            None => (built_in.colour, finish_of(built_in.finish), Outline::Auto),
        };
        let [hue, sat, light] = hsl(colour);
        CartEdit {
            stem: cart.stem.clone(),
            cart: cart.clone(),
            start: cart.shell,
            default: (built_in, Outline::Auto),
            presets: shell_presets(),
            cursor: 0,
            hue,
            sat,
            light,
            finish,
            outline,
            use_default: cart.shell.is_none(),
            picture_wanted: false,
            source: PictureSource::Gallery,
            picking: None,
            restore_wanted: false,
            text_wanted: false,
            text_note: "Type",
            picture_note: "",
            picture_changed: false,
            version: 0,
        }
    }

    pub fn rows(&self) -> Vec<EditRow> {
        let mut rows = vec![
            EditRow::Preset,
            EditRow::Hue,
            EditRow::Vividness,
            EditRow::Shade,
            EditRow::Finish,
        ];
        if self.cart.platform != Platform::Gba {
            rows.push(EditRow::Outline);
        }
        rows.push(EditRow::Picture);
        rows.push(EditRow::Text);
        rows.push(EditRow::Reset);
        rows
    }

    fn colour(&self) -> [u8; 3] {
        rgb(self.hue, self.sat, self.light)
    }

    pub fn choice(&self) -> ShellChoice {
        ShellChoice {
            outline: self.outline,
            colour: self.colour(),
            finish: self.finish,
        }
    }

    /// What to remember: `None` when the cart is back on the built-in colour.
    pub fn result(&self) -> Option<ShellChoice> {
        (!self.use_default).then(|| self.choice())
    }

    pub fn changed(&self) -> bool {
        self.result() != self.start
    }

    /// The cart as it would be drawn right now.
    pub fn preview_cart(&self) -> Cart {
        let mut cart = self.cart.clone();
        cart.shell = Some(self.choice());
        cart
    }

    fn preset_index(&self) -> Option<usize> {
        let (colour, finish) = (self.colour(), finish_ui(self.finish));
        self.presets
            .iter()
            .position(|(_, s)| s.colour == colour && s.finish == finish)
    }

    pub fn step(&mut self, dir: i32) {
        let rows = self.rows();
        let Some(row) = rows.get(self.cursor).copied() else {
            return;
        };
        let mut moved = true;
        match row {
            EditRow::Preset => {
                let n = self.presets.len() as i32;
                let at = self.preset_index().map_or(if dir > 0 { -1 } else { 0 }, |i| i as i32);
                let shell = self.presets[(at + dir).rem_euclid(n) as usize].1;
                let [h, s, l] = hsl(shell.colour);
                (self.hue, self.sat, self.light) = (h, s, l);
                self.finish = finish_of(shell.finish);
            }
            EditRow::Hue => {
                if self.sat < 0.05 {
                    self.sat = 0.6;
                }
                self.hue = (self.hue + 10.0 * dir as f32).rem_euclid(360.0);
            }
            EditRow::Vividness => self.sat = (self.sat + 0.1 * dir as f32).clamp(0.0, 1.0),
            EditRow::Shade => self.light = (self.light + 0.05 * dir as f32).clamp(0.1, 0.9),
            EditRow::Finish => {
                const ORDER: [ShellFinish; 3] =
                    [ShellFinish::Solid, ShellFinish::Clear, ShellFinish::Glitter];
                let at = ORDER.iter().position(|f| *f == self.finish).unwrap_or(0) as i32;
                self.finish = ORDER[(at + dir).rem_euclid(3) as usize];
            }
            EditRow::Outline => {
                const ORDER: [Outline; 3] = [Outline::Auto, Outline::Notched, Outline::Rounded];
                let at = ORDER.iter().position(|o| *o == self.outline).unwrap_or(0) as i32;
                self.outline = ORDER[(at + dir).rem_euclid(3) as usize];
            }
            EditRow::Text => {
                self.text_wanted = true;
                moved = false;
            }
            EditRow::Picture => {
                self.source = match self.source {
                    PictureSource::Gallery => PictureSource::Downloads,
                    PictureSource::Downloads => PictureSource::Gallery,
                };
                self.picture_note = "";
                moved = false;
            }
            EditRow::Reset => {
                self.reset();
                self.restore_wanted = true;
                moved = false;
            }
        }
        if moved {
            self.use_default = false;
        }
        self.version = self.version.wrapping_add(1);
    }

    /// A (or a tap) on the current row: the rows that run something do it, the rest step.
    pub fn activate(&mut self) {
        match self.rows().get(self.cursor) {
            Some(EditRow::Picture) => {
                if self.picking.is_none() {
                    self.picture_wanted = true;
                }
                self.version = self.version.wrapping_add(1);
            }
            Some(EditRow::Text) => {
                self.text_wanted = true;
                self.version = self.version.wrapping_add(1);
            }
            _ => self.step(1),
        }
    }

    /// Whether the player asked to type the label's words (reads once).
    pub fn take_text_request(&mut self) -> bool {
        std::mem::take(&mut self.text_wanted)
    }

    pub fn text_done(&mut self, label: Result<std::path::PathBuf, &'static str>) {
        match label {
            Ok(path) => {
                self.cart.label = Some(path);
                self.text_note = "Set";
                self.picture_changed = true;
            }
            Err(note) => self.text_note = note,
        }
        self.version = self.version.wrapping_add(1);
    }

    pub fn source(&self) -> PictureSource {
        self.source
    }

    pub fn start_picking(&mut self, dest: std::path::PathBuf) {
        self.picking = Some(dest);
        self.picture_note = "Waiting";
        self.version = self.version.wrapping_add(1);
    }

    /// Whether the player asked for a picture since last asked (taken: it reads once).
    pub fn take_picture_request(&mut self) -> bool {
        std::mem::take(&mut self.picture_wanted)
    }

    /// Whether "Back to default" asked for the original label to come back (reads once).
    pub fn take_restore_request(&mut self) -> bool {
        std::mem::take(&mut self.restore_wanted)
    }

    /// The label the cart has again after the original was put back.
    pub fn label_restored(&mut self, label: Option<std::path::PathBuf>) {
        self.cart.label = label;
        self.picture_note = "";
        self.text_note = "Type";
        self.picture_changed = true;
        self.version = self.version.wrapping_add(1);
    }

    pub fn platform(&self) -> Platform {
        self.cart.platform
    }

    /// The outcome of a picture request: the new label file, or what went wrong.
    pub fn picture_done(&mut self, label: Result<std::path::PathBuf, &'static str>) {
        self.picking = None;
        match label {
            Ok(path) => {
                self.cart.label = Some(path);
                self.picture_note = "Set";
                self.picture_changed = true;
            }
            Err(note) => self.picture_note = note,
        }
        self.version = self.version.wrapping_add(1);
    }

    pub fn reset(&mut self) {
        let [h, s, l] = hsl(self.default.0.colour);
        (self.hue, self.sat, self.light) = (h, s, l);
        self.finish = finish_of(self.default.0.finish);
        self.outline = self.default.1;
        self.use_default = true;
        self.version = self.version.wrapping_add(1);
    }

    pub fn move_cursor(&mut self, dir: i32) {
        let n = self.rows().len() as i32;
        self.cursor = (self.cursor as i32 + dir).rem_euclid(n) as usize;
        self.version = self.version.wrapping_add(1);
    }

    /// `(label, value)` for every row.
    pub fn lines(&self) -> Vec<(&'static str, String)> {
        self.rows()
            .into_iter()
            .map(|row| {
                let value = match row {
                    EditRow::Preset => match self.preset_index() {
                        Some(i) => self.presets[i].0.to_string(),
                        None if self.use_default => "Built in".to_string(),
                        None => "Custom".to_string(),
                    },
                    EditRow::Hue => format!("{}°", self.hue.round() as i32 % 360),
                    EditRow::Vividness => format!("{}%", (self.sat * 100.0).round() as i32),
                    EditRow::Shade => format!("{}%", (self.light * 100.0).round() as i32),
                    EditRow::Finish => match self.finish {
                        ShellFinish::Solid => "Solid",
                        ShellFinish::Clear => "Clear",
                        ShellFinish::Glitter => "Glitter",
                    }
                    .to_string(),
                    EditRow::Outline => match self.outline {
                        Outline::Auto => "Auto",
                        Outline::Notched => "Notched",
                        Outline::Rounded => "Rounded",
                    }
                    .to_string(),
                    EditRow::Picture => {
                        if self.picture_note.is_empty() {
                            match self.source {
                                PictureSource::Gallery => "Gallery",
                                PictureSource::Downloads => "Downloads",
                            }
                            .to_string()
                        } else {
                            self.picture_note.to_string()
                        }
                    }
                    EditRow::Text => self.text_note.to_string(),
                    EditRow::Reset => String::new(),
                };
                (row.label(), value)
            })
            .collect()
    }
}

fn finish_of(f: Finish) -> ShellFinish {
    match f {
        Finish::Solid => ShellFinish::Solid,
        Finish::Translucent => ShellFinish::Clear,
        Finish::Glitter => ShellFinish::Glitter,
    }
}

fn finish_ui(f: ShellFinish) -> Finish {
    match f {
        ShellFinish::Solid => Finish::Solid,
        ShellFinish::Clear => Finish::Translucent,
        ShellFinish::Glitter => Finish::Glitter,
    }
}

fn hsl([r, g, b]: [u8; 3]) -> [f32; 3] {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    if d < 1e-4 {
        return [0.0, 0.0, l];
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    [h * 60.0, s.clamp(0.0, 1.0), l]
}

fn rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h.rem_euclid(360.0) / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r, g, b].map(|v| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8)
}

#[derive(Default)]
pub(crate) struct CartEditPanel {
    texture: Option<TexId>,
    preview: Option<(TexId, u32, u32)>,
    built_for: Option<(String, u32)>,
    visible: bool,
    cursor: usize,
}

impl CartEditPanel {
    pub fn update(&mut self, session: &Session, compositor: &mut Compositor) {
        let app = session.app();
        if matches!(app.phase(), Phase::Doze { .. }) {
            self.visible = false;
            return;
        }
        let Some((edit, root)) = app.cart_edit_view() else {
            self.visible = false;
            self.built_for = None;
            return;
        };
        self.visible = true;
        self.cursor = edit.cursor;
        let key = (edit.stem.clone(), edit.version);
        if self.built_for.as_ref() == Some(&key) && self.texture.is_some() {
            return;
        }
        let face = build(edit);
        match self.texture {
            Some(id) => compositor.update_texture(id, face.w, face.h, &face.rgba),
            None => self.texture = Some(compositor.create_texture(face.w, face.h, &face.rgba)),
        }
        let cart = edit.preview_cart();
        let art = root.and_then(|r| label_art(r, &cart));
        let shown = cart_face_with(&cart, art);
        match self.preview {
            Some((id, ..)) => compositor.update_texture(id, shown.w, shown.h, &shown.rgba),
            None => {
                let id = compositor.create_texture(shown.w, shown.h, &shown.rgba);
                self.preview = Some((id, shown.w, shown.h));
            }
        }
        if let Some(p) = self.preview.as_mut() {
            p.1 = shown.w;
            p.2 = shown.h;
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
        draws.push(Draw::Rect {
            x: 0.0,
            y: ROWS_TOP + PITCH * self.cursor as f32 + 4.0,
            w: COLUMN_W + 16.0,
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
        if let Some((id, w, h)) = self.preview {
            let (bx, by, bw, bh) = PREVIEW;
            let scale = (bw / w as f32).min(bh / h as f32);
            let (dw, dh) = ((w as f32 * scale).round(), (h as f32 * scale).round());
            draws.push(Draw::Tex {
                x: (bx + (bw - dw) / 2.0).round(),
                y: (by + (bh - dh) / 2.0).round(),
                w: dw,
                h: dh,
                tex: id,
                alpha: 1.0,
            });
        }
    }
}

fn build(edit: &CartEdit) -> UndoFace {
    let (w, h) = (OUT_W, OUT_H);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let _ = text::label_font();
    line(&mut rgba, "Edit Cart", EDGE, TITLE_Y, INK, false);
    let name = slot_ui::clean_label(&edit.stem);
    line_w(&mut rgba, &name, w - EDGE, TITLE_Y, DIM, true, 280.0);
    for (i, (label, value)) in edit.lines().iter().enumerate() {
        let y = ROWS_TOP as u32 + PITCH as u32 * i as u32 + 2;
        let lit = i == edit.cursor;
        let ink = if lit { INK } else { DIM };
        if *label == EditRow::Preset.label() {
            // A preset's name is too long to sit opposite its label, so they share the line.
            let both = format!("Preset: {value}");
            line_w(&mut rgba, &both, EDGE, y, ink, false, COLUMN_W - EDGE as f32 - 8.0);
            continue;
        }
        line(&mut rgba, label, EDGE, y, ink, false);
        if !value.is_empty() {
            line(&mut rgba, value, COLUMN_W as u32 - 8, y, INK, true);
        }
    }
    for (x, face) in [
        (EDGE, hint_face("B", "Done")),
        (w / 2, hint_face("A", "Change")),
    ] {
        blit(&mut rgba, w, &face, x, LEGEND_Y);
    }
    UndoFace { rgba, w, h }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_survives_the_round_trip_through_hue_saturation_and_lightness() {
        for c in [[0xc0, 0x28, 0x2c], [0x35, 0x35, 0x3a], [0xec, 0x94, 0xb4], [255, 255, 255]] {
            let [h, s, l] = hsl(c);
            let back = rgb(h, s, l);
            for i in 0..3 {
                assert!((back[i] as i32 - c[i] as i32).abs() <= 1, "{c:?} -> {back:?}");
            }
        }
    }
}
