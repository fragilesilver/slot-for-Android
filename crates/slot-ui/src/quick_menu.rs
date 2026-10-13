use crate::draw::{Draw, TexId, OUT_H, OUT_W};
use crate::plate::{arrows_hint_face, centred_hints, hint_face, UndoFace, HINT_H, LEGEND_GAP};
use crate::power_menu::{MENU_H, MENU_INK, MENU_PAD, MENU_PX};
use crate::slot_chrome::{edge, opening};
use crate::text;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickRow {
    FastForward,
    FastForwardSound,
    Screen,
    Rumble,
    DateTime,
    About,
    GbaShader,
    GbShader,
    ColourCorrection,
    Game,
    EjectSave,
    Turbo,
    Rewind,
    GbPalettes,
    SaveState,
    LoadState,
    Core,
    Shelf,
    States,
    Controls,
    TurboButtons,
    XButton,
    YButton,
    FrameBlending,
    Cheats,
    ColourGrade,
    ShaderStrength,
    /// The pause menu's own "Screen" row (the shelf menu's `Screen` lives on its MAIN page).
    PauseScreen,
    Favourite,
    FavouritesOnly,
    TouchOpacity,
    TouchSize,
    Achievements,
    PauseAchievements,
    EditCart,
    Backup,
    TouchHaptics,
    Vibration,
    TouchMode,
}

impl QuickRow {
    pub const ALL: [QuickRow; 39] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Screen,
        QuickRow::Rumble,
        QuickRow::DateTime,
        QuickRow::About,
        QuickRow::GbaShader,
        QuickRow::GbShader,
        QuickRow::ColourCorrection,
        QuickRow::Game,
        QuickRow::EjectSave,
        QuickRow::Turbo,
        QuickRow::Rewind,
        QuickRow::GbPalettes,
        QuickRow::SaveState,
        QuickRow::LoadState,
        QuickRow::Core,
        QuickRow::Shelf,
        QuickRow::States,
        QuickRow::Controls,
        QuickRow::TurboButtons,
        QuickRow::XButton,
        QuickRow::YButton,
        QuickRow::FrameBlending,
        QuickRow::Cheats,
        QuickRow::ColourGrade,
        QuickRow::ShaderStrength,
        QuickRow::PauseScreen,
        QuickRow::Favourite,
        QuickRow::FavouritesOnly,
        QuickRow::TouchOpacity,
        QuickRow::TouchSize,
        QuickRow::Achievements,
        QuickRow::PauseAchievements,
        QuickRow::EditCart,
        QuickRow::Backup,
        QuickRow::TouchHaptics,
        QuickRow::Vibration,
        QuickRow::TouchMode,
    ];

    /// The in-game pause menu (Android port): opened over a running game.
    pub const PAUSE: [QuickRow; 9] = [
        QuickRow::SaveState,
        QuickRow::LoadState,
        QuickRow::States,
        QuickRow::Core,
        QuickRow::Controls,
        QuickRow::PauseScreen,
        QuickRow::Cheats,
        QuickRow::PauseAchievements,
        QuickRow::Shelf,
    ];

    /// The pause menu's Controls page.
    pub const CONTROLS: [QuickRow; 8] = [
        QuickRow::TurboButtons,
        QuickRow::XButton,
        QuickRow::YButton,
        QuickRow::TouchMode,
        QuickRow::TouchOpacity,
        QuickRow::TouchSize,
        QuickRow::TouchHaptics,
        QuickRow::Vibration,
    ];

    pub const MAIN: [QuickRow; 9] = [
        QuickRow::Favourite,
        QuickRow::FavouritesOnly,
        QuickRow::EditCart,
        QuickRow::Screen,
        QuickRow::Game,
        QuickRow::Achievements,
        QuickRow::Backup,
        QuickRow::DateTime,
        QuickRow::About,
    ];

    pub const SCREEN: [QuickRow; 7] = [
        QuickRow::GbaShader,
        QuickRow::GbShader,
        QuickRow::ShaderStrength,
        QuickRow::ColourGrade,
        QuickRow::ColourCorrection,
        QuickRow::FrameBlending,
        QuickRow::GbPalettes,
    ];

    pub const GAME: [QuickRow; 6] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Rewind,
        QuickRow::Turbo,
        QuickRow::Rumble,
        QuickRow::EjectSave,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn page(self) -> &'static [QuickRow] {
        match self {
            QuickRow::GbaShader
            | QuickRow::GbShader
            | QuickRow::ColourCorrection
            | QuickRow::FrameBlending
            | QuickRow::ShaderStrength
            | QuickRow::ColourGrade
            | QuickRow::GbPalettes => &QuickRow::SCREEN,
            QuickRow::FastForward
            | QuickRow::FastForwardSound
            | QuickRow::Rewind
            | QuickRow::Turbo
            | QuickRow::Rumble
            | QuickRow::EjectSave => &QuickRow::GAME,
            QuickRow::SaveState
            | QuickRow::LoadState
            | QuickRow::States
            | QuickRow::Core
            | QuickRow::Controls
            | QuickRow::PauseScreen
            | QuickRow::Cheats
            | QuickRow::PauseAchievements
            | QuickRow::Shelf => &QuickRow::PAUSE,
            QuickRow::TurboButtons
            | QuickRow::XButton
            | QuickRow::YButton
            | QuickRow::TouchOpacity
            | QuickRow::TouchSize
            | QuickRow::TouchHaptics
            | QuickRow::TouchMode
            | QuickRow::Vibration => &QuickRow::CONTROLS,
            _ => &QuickRow::MAIN,
        }
    }

    pub fn parent(self) -> Option<QuickRow> {
        match self.page()[0] {
            QuickRow::GbaShader => Some(QuickRow::Screen),
            QuickRow::FastForward => Some(QuickRow::Game),
            QuickRow::TurboButtons => Some(QuickRow::Controls),
            _ => None,
        }
    }

    pub fn child(self) -> Option<QuickRow> {
        match self {
            QuickRow::Screen => Some(QuickRow::SCREEN[0]),
            QuickRow::Game => Some(QuickRow::GAME[0]),
            QuickRow::Controls => Some(QuickRow::CONTROLS[0]),
            QuickRow::PauseScreen => Some(QuickRow::SCREEN[0]),
            _ => None,
        }
    }

    pub fn position(self) -> usize {
        self.page().iter().position(|r| *r == self).unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            QuickRow::FastForward => "Fast Forward",
            QuickRow::FastForwardSound => "Fast Forward Sound",
            QuickRow::Screen => "Screen",
            QuickRow::Rumble => "Rumble",
            QuickRow::DateTime => "Date & Time",
            QuickRow::About => "About",
            QuickRow::GbaShader => "GBA Shader",
            QuickRow::GbShader => "GB / GBC Shader",
            QuickRow::ColourCorrection => "Color Correction",
            QuickRow::Game => "Gameplay",
            QuickRow::EjectSave => "Auto Save on Eject",
            QuickRow::Turbo => "Turbo Buttons",
            QuickRow::Rewind => "Rewind",
            QuickRow::GbPalettes => "GB Palettes",
            QuickRow::SaveState => "Save State",
            QuickRow::LoadState => "Load State",
            QuickRow::Core => "Core",
            QuickRow::Shelf => "Back to Shelf",
            QuickRow::States => "Browse States",
            QuickRow::Controls => "Controls",
            QuickRow::TurboButtons => "Turbo Buttons",
            QuickRow::XButton => "X Button",
            QuickRow::YButton => "Y Button",
            QuickRow::FrameBlending => "Frame Blending",
            QuickRow::Cheats => "Cheats",
            QuickRow::ColourGrade => "Color Grade",
            QuickRow::ShaderStrength => "Shader Strength",
            QuickRow::PauseScreen => "Screen",
            QuickRow::Favourite => "Favourite",
            QuickRow::FavouritesOnly => "Favourites Only",
            QuickRow::TouchOpacity => "Controls Opacity",
            QuickRow::TouchSize => "Controls Size",
            QuickRow::Achievements | QuickRow::PauseAchievements => "Achievements",
            QuickRow::EditCart => "Edit Cart",
            QuickRow::Backup => "Library",
            QuickRow::TouchHaptics => "Touch Haptics",
            QuickRow::Vibration => "Vibration",
            QuickRow::TouchMode => "Touch Controls",
        }
    }

    pub fn opens(self) -> bool {
        matches!(
            self,
            QuickRow::DateTime
                | QuickRow::About
                | QuickRow::Screen
                | QuickRow::Game
                | QuickRow::SaveState
                | QuickRow::LoadState
                | QuickRow::States
                | QuickRow::Controls
                | QuickRow::PauseScreen
                | QuickRow::Cheats
                | QuickRow::Achievements
                | QuickRow::PauseAchievements
                | QuickRow::EditCart
                | QuickRow::Backup
                | QuickRow::Shelf
        )
    }

    pub fn up(self) -> QuickRow {
        let page = self.page();
        page[(self.position() + page.len() - 1) % page.len()]
    }

    pub fn down(self) -> QuickRow {
        let page = self.page();
        page[(self.position() + 1) % page.len()]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickValue {
    Speed2,
    Speed3,
    Speed4,
    Speed6,
    On,
    Off,
    Lcd3x,
    Grid,
    Dot,
    Simpletex,
    Mgba,
    Gpsp,
    Turbo,
    ButtonA,
    ButtonB,
    GradeFull,
    GradeHalf,
    GradeMono,
    GradeDmg,
    GradeAmber,
    GradeIce,
    GradePink,
    Percent50,
    Percent100,
    Percent30,
    Percent70,
    Percent80,
    Percent120,
    Auto,
}

impl QuickValue {
    pub const ALL: [QuickValue; 29] = [
        QuickValue::Speed2,
        QuickValue::Speed3,
        QuickValue::Speed4,
        QuickValue::Speed6,
        QuickValue::On,
        QuickValue::Off,
        QuickValue::Lcd3x,
        QuickValue::Grid,
        QuickValue::Dot,
        QuickValue::Simpletex,
        QuickValue::Mgba,
        QuickValue::Gpsp,
        QuickValue::Turbo,
        QuickValue::ButtonA,
        QuickValue::ButtonB,
        QuickValue::GradeFull,
        QuickValue::GradeHalf,
        QuickValue::GradeMono,
        QuickValue::GradeDmg,
        QuickValue::GradeAmber,
        QuickValue::GradeIce,
        QuickValue::GradePink,
        QuickValue::Percent50,
        QuickValue::Percent100,
        QuickValue::Percent30,
        QuickValue::Percent70,
        QuickValue::Percent80,
        QuickValue::Percent120,
        QuickValue::Auto,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            QuickValue::Speed2 => "2×",
            QuickValue::Speed3 => "3×",
            QuickValue::Speed4 => "4×",
            QuickValue::Speed6 => "6×",
            QuickValue::On => "On",
            QuickValue::Off => "Off",
            QuickValue::Lcd3x => "LCD3x",
            QuickValue::Grid => "Grid",
            QuickValue::Dot => "Dot",
            QuickValue::Simpletex => "Simpletex",
            QuickValue::Mgba => "mGBA",
            QuickValue::Gpsp => "gpSP",
            QuickValue::Turbo => "Turbo",
            QuickValue::ButtonA => "A",
            QuickValue::ButtonB => "B",
            QuickValue::GradeFull => "Full Color",
            QuickValue::GradeHalf => "Half Color",
            QuickValue::GradeMono => "No Color",
            QuickValue::GradeDmg => "DMG Green",
            QuickValue::GradeAmber => "Amber",
            QuickValue::GradeIce => "Ice Blue",
            QuickValue::GradePink => "Pink",
            QuickValue::Percent50 => "50%",
            QuickValue::Percent100 => "100%",
            QuickValue::Percent30 => "30%",
            QuickValue::Percent70 => "70%",
            QuickValue::Percent80 => "80%",
            QuickValue::Percent120 => "120%",
            QuickValue::Auto => "Auto",
        }
    }

    /// The value that prints as `percent`, for the percentages the menus step through.
    pub fn percent(percent: u8) -> QuickValue {
        match percent {
            30 => QuickValue::Percent30,
            50 => QuickValue::Percent50,
            70 => QuickValue::Percent70,
            80 => QuickValue::Percent80,
            120 => QuickValue::Percent120,
            _ => QuickValue::Percent100,
        }
    }

    pub fn grade(index: u8) -> QuickValue {
        match index {
            1 => QuickValue::GradeHalf,
            2 => QuickValue::GradeMono,
            3 => QuickValue::GradeDmg,
            4 => QuickValue::GradeAmber,
            5 => QuickValue::GradeIce,
            6 => QuickValue::GradePink,
            _ => QuickValue::GradeFull,
        }
    }

    pub fn speed(frames: u8) -> Option<QuickValue> {
        match frames {
            2 => Some(QuickValue::Speed2),
            3 => Some(QuickValue::Speed3),
            4 => Some(QuickValue::Speed4),
            6 => Some(QuickValue::Speed6),
            _ => None,
        }
    }

    pub fn flag(on: bool) -> QuickValue {
        if on {
            QuickValue::On
        } else {
            QuickValue::Off
        }
    }
}

pub const QUICK_PITCH: f32 = 52.0;
pub const QUICK_TOP: f32 = (OUT_H as f32 - QUICK_PITCH * QuickRow::MAIN.len() as f32) / 2.0;
pub const QUICK_EDGE: f32 = 32.0;
const BAR_INSET: f32 = 4.0;
const TYPE_DROP: f32 = 4.0;
const CARET_GAP: f32 = 14.0;
const CARET_PX: f32 = 24.0;
const LEGEND_Y: f32 = 427.0;
pub const QUICK_SPLIT: f32 = LEGEND_Y - 2.0;
const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];

pub fn quick_label_face(row: QuickRow) -> UndoFace {
    quick_text_face(row.label(), MENU_INK)
}

pub fn quick_value_face(text: &str, lit: bool) -> UndoFace {
    quick_text_face(text, if lit { MENU_INK } else { DIM_INK })
}

fn quick_text_face(label: &str, colour: [u8; 3]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::fit(font, label, OUT_W as f32, 1, MENU_PX, MENU_PX);
    let set = layout
        .lines
        .iter()
        .map(|l| text::line_width(font, l, layout.px, layout.tracking))
        .fold(0.0, f32::max);
    let w = set.ceil() as u32 + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    text::draw_centred(&mut rgba, w, MENU_H, &layout, colour);
    UndoFace { rgba, w, h: MENU_H }
}

pub fn quick_caret_face(right: bool) -> UndoFace {
    let glyph = if right { '\u{f0da}' } else { '\u{f0d9}' };
    let (Some(symbols), Some(label)) = (crate::icon::symbols_font(), text::label_font()) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let (m, cov) = symbols.rasterize(glyph, CARET_PX);
    let (w, h) = (m.width as u32, MENU_H);
    let centre = match label.horizontal_line_metrics(MENU_PX) {
        Some(v) => {
            (h as f32 - v.new_line_size) / 2.0 + v.ascent
                - label.metrics('H', MENU_PX).height as f32 / 2.0
        }
        None => h as f32 / 2.0,
    };
    let top = (centre - m.height as f32 / 2.0).round() as i32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for gy in 0..m.height {
        let y = top + gy as i32;
        if y < 0 || y >= h as i32 {
            continue;
        }
        for gx in 0..m.width {
            let at = ((y as u32 * w + gx as u32) * 4) as usize;
            let a = cov[gy * m.width + gx];
            rgba[at..at + 4].copy_from_slice(&[MENU_INK[0], MENU_INK[1], MENU_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

pub fn quick_legend_faces() -> [UndoFace; 3] {
    [
        hint_face("B", "Back"),
        arrows_hint_face("Change"),
        hint_face("A", "Open"),
    ]
}

pub struct QuickMenuFaces {
    pub labels: Vec<(TexId, u32, u32)>,
    pub values: Vec<[(TexId, u32, u32); 2]>,
    pub carets: [(TexId, u32, u32); 2],
    pub legend: [(TexId, u32); 3],
}

pub struct QuickMenu<'a> {
    pub row: QuickRow,
    pub values: [Option<QuickValue>; QuickRow::ALL.len()],
    pub clock: Option<[(TexId, u32, u32); 2]>,
    /// The GB Palettes row's value: a palette's name, which the fixed values cannot hold.
    pub palette: Option<[(TexId, u32, u32); 2]>,
    pub faces: Option<&'a QuickMenuFaces>,
}

impl QuickMenu<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: OUT_W as f32,
            h: OUT_H as f32,
            colour: opening(),
        });
        out.push(Draw::Rect {
            x: 0.0,
            y: row_top(self.row) + BAR_INSET,
            w: OUT_W as f32,
            h: pitch_of(self.row) - 2.0 * BAR_INSET,
            colour: edge(),
        });
        let Some(faces) = self.faces else {
            return;
        };
        let (right, pad) = (OUT_W as f32 - QUICK_EDGE, MENU_PAD as f32);
        for &row in self.row.page() {
            let y = row_top(row) + TYPE_DROP;
            let lit = row == self.row;
            if let Some(&(tex, w, h)) = faces.labels.get(row.index()) {
                push(out, tex, QUICK_EDGE - pad, y, w, h);
            }
            let value = match row {
                QuickRow::DateTime => self.clock.map(|c| c[lit as usize]),
                QuickRow::GbPalettes => self.palette.map(|c| c[lit as usize]),
                _ => self.values[row.index()]
                    .and_then(|v| faces.values.get(v.index()))
                    .map(|v| v[lit as usize]),
            };
            let Some((tex, w, h)) = value else {
                continue;
            };
            if !lit || row.opens() {
                push(out, tex, right + pad - w as f32, y, w, h);
                continue;
            }
            let [(left_tex, lw, lh), (right_tex, rw, rh)] = faces.carets;
            let rx = right - rw as f32;
            push(out, right_tex, rx, y, rw, rh);
            let vx = rx - CARET_GAP + pad - w as f32;
            push(out, tex, vx, y, w, h);
            push(out, left_tex, vx + pad - CARET_GAP - lw as f32, y, lw, lh);
        }
        let [back, change, open] = faces.legend;
        let other = if self.row.opens() { open } else { change };
        for (tex, w, x) in centred_hints(&[back, other], LEGEND_GAP) {
            push(out, tex, x, LEGEND_Y, w, HINT_H);
        }
    }
}

/// Rows are spaced `QUICK_PITCH` apart, squeezed only when a page is too long to clear the legend.
fn pitch_of(row: QuickRow) -> f32 {
    QUICK_PITCH.min(368.0 / row.page().len() as f32)
}

fn row_top(row: QuickRow) -> f32 {
    let pitch = pitch_of(row);
    let top = (OUT_H as f32 - pitch * row.page().len() as f32) / 2.0;
    top + pitch * row.position() as f32
}

fn push(out: &mut Vec<Draw>, tex: TexId, x: f32, y: f32, w: u32, h: u32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}
