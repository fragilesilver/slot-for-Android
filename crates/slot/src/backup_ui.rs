//! The Backup screen: two actions, the newest backup's name and the outcome of the last action.
use std::path::PathBuf;

use slot_gfx::{Compositor, Draw, TexId, OUT_H, OUT_W};
use slot_ui::{edge, hint_face, opening, UndoFace};

use crate::app::Phase;
use crate::cheat_ui::{blit, line};
use crate::session::Session;

const TITLE_Y: u32 = 22;
const EDGE: u32 = 32;
const ROWS_TOP: f32 = 84.0;
const PITCH: f32 = 52.0;
const LEGEND_Y: u32 = 427;
const INK: [u8; 3] = [0xf6, 0xf4, 0xef];
const DIM: [u8; 3] = [0x9a, 0x9a, 0xa4];
const LIT: [u8; 3] = [0x7d, 0xe0, 0x9a];
const ROWS: [&str; 3] = ["Back up now", "Restore latest backup", "Import games from Downloads"];

pub(crate) struct BackupState {
    pub cursor: usize,
    latest: Option<String>,
    message: Option<String>,
    version: u32,
}

pub(crate) enum Tapped {
    Row(usize),
    Back,
    Nothing,
}

pub(crate) fn tap(x: f32, y: f32) -> Tapped {
    if y >= LEGEND_Y as f32 - 12.0 && x < OUT_W as f32 / 2.0 {
        return Tapped::Back;
    }
    if y >= ROWS_TOP {
        let row = ((y - ROWS_TOP) / PITCH) as usize;
        if row < ROWS.len() {
            return Tapped::Row(row);
        }
    }
    Tapped::Nothing
}

fn name_of(p: &std::path::Path) -> Option<String> {
    Some(p.file_name()?.to_str()?.to_string())
}

impl BackupState {
    pub fn new(latest: Option<PathBuf>) -> Self {
        BackupState {
            cursor: 0,
            latest: latest.as_deref().and_then(name_of),
            message: None,
            version: 0,
        }
    }

    pub fn move_cursor(&mut self, dir: i32) {
        self.cursor = (self.cursor as i32 + dir).rem_euclid(ROWS.len() as i32) as usize;
        self.version = self.version.wrapping_add(1);
    }

    pub fn finish(&mut self, message: String, latest: Option<PathBuf>) {
        self.message = Some(message);
        self.latest = latest.as_deref().and_then(name_of);
        self.version = self.version.wrapping_add(1);
    }
}

#[derive(Default)]
pub(crate) struct BackupPanel {
    texture: Option<TexId>,
    built_for: Option<u32>,
    visible: bool,
    cursor: usize,
}

impl BackupPanel {
    pub fn update(&mut self, session: &Session, compositor: &mut Compositor) {
        let app = session.app();
        if matches!(app.phase(), Phase::Doze { .. }) {
            self.visible = false;
            return;
        }
        let Some(state) = app.backup_view() else {
            self.visible = false;
            self.built_for = None;
            return;
        };
        self.visible = true;
        self.cursor = state.cursor;
        if self.built_for == Some(state.version) && self.texture.is_some() {
            return;
        }
        let face = build(state);
        match self.texture {
            Some(id) => compositor.update_texture(id, face.w, face.h, &face.rgba),
            None => self.texture = Some(compositor.create_texture(face.w, face.h, &face.rgba)),
        }
        self.built_for = Some(state.version);
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

fn build(state: &BackupState) -> UndoFace {
    let (w, h) = (OUT_W, OUT_H);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    line(&mut rgba, "Library", EDGE, TITLE_Y, INK, false);
    for (i, label) in ROWS.iter().enumerate() {
        let y = ROWS_TOP as u32 + PITCH as u32 * i as u32 + 8;
        let ink = if i == state.cursor { INK } else { DIM };
        line(&mut rgba, label, EDGE, y, ink, false);
    }
    let base = ROWS_TOP as u32 + PITCH as u32 * ROWS.len() as u32 + 24;
    let latest = match &state.latest {
        Some(n) => format!("Latest: {n}"),
        None => "No backup yet".to_string(),
    };
    line(&mut rgba, &latest, EDGE, base, DIM, false);
    line(&mut rgba, "Backups keep saves, states and cart colours", EDGE, base + 40, DIM, false);
    if let Some(m) = &state.message {
        line(&mut rgba, m, EDGE, base + 80, LIT, false);
    }
    for (x, face) in [(EDGE, hint_face("B", "Back")), (w / 2, hint_face("A", "Run"))] {
        blit(&mut rgba, w, &face, x, LEGEND_Y);
    }
    UndoFace { rgba, w, h }
}
