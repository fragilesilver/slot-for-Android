use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::gb_palette::GbPalette;
use crate::platform::Platform;

pub const BRIGHTNESS_MAX: u8 = 9;
pub const BLUE_LIGHT_MAX: u8 = 9;
pub const VOLUME_MAX: u8 = 100;

pub const UTC_OFFSET_MIN: i16 = -720;
pub const UTC_OFFSET_MAX: i16 = 840;

pub const FF_SPEEDS: [u8; 4] = [2, 3, 4, 6];

pub const FF_SPEED_DEFAULT: u8 = 6;

/// When the on-screen controls show: `Auto` hides them while a pad or keyboard is in use and
/// brings them back on a touch; `On` and `Off` pin them.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TouchMode {
    Auto,
    On,
    Off,
}

impl TouchMode {
    pub fn name(self) -> &'static str {
        match self {
            TouchMode::Auto => "auto",
            TouchMode::On => "on",
            TouchMode::Off => "off",
        }
    }

    pub fn parse(s: &str) -> Option<TouchMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(TouchMode::Auto),
            "on" => Some(TouchMode::On),
            "off" => Some(TouchMode::Off),
            _ => None,
        }
    }

    pub fn step(self, right: bool) -> TouchMode {
        const ALL: [TouchMode; 3] = [TouchMode::Auto, TouchMode::On, TouchMode::Off];
        let at = ALL.iter().position(|m| *m == self).unwrap_or(0);
        ALL[if right { (at + 1) % 3 } else { (at + 2) % 3 }]
    }
}

/// What an on-screen X / Y button does: slot's own (hold X for turbo A, Y for turbo B), or
/// press A or B instead.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ButtonRole {
    Turbo,
    A,
    B,
}

impl ButtonRole {
    pub fn name(self) -> &'static str {
        match self {
            ButtonRole::Turbo => "turbo",
            ButtonRole::A => "a",
            ButtonRole::B => "b",
        }
    }

    pub fn parse(s: &str) -> Option<ButtonRole> {
        match s.trim().to_ascii_lowercase().as_str() {
            "turbo" => Some(ButtonRole::Turbo),
            "a" => Some(ButtonRole::A),
            "b" => Some(ButtonRole::B),
            _ => None,
        }
    }

    pub fn step(self, right: bool) -> ButtonRole {
        const ALL: [ButtonRole; 3] = [ButtonRole::Turbo, ButtonRole::A, ButtonRole::B];
        let at = ALL.iter().position(|r| *r == self).unwrap_or(0);
        ALL[if right { (at + 1) % 3 } else { (at + 2) % 3 }]
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SlotState {
    pub cart: Option<String>,
    pub cart_platform: Option<Platform>,
    pub brightness: u8,
    pub blue_light: u8,
    pub volume: u8,
    pub volume_hp: u8,
    pub muted: bool,
    pub muted_hp: bool,
    pub clock_set: bool,
    pub utc_offset_min: i16,
    pub rumble: bool,
    pub ff_speed: u8,
    pub ff_sound: bool,
    pub colour_correction: bool,
    pub shader_gba: Shader,
    pub shader_gb: Shader,
    pub eject_save: bool,
    pub turbo: bool,
    pub rewind: bool,
    pub gb_palettes: bool,
    pub gb_palette: GbPalette,
    pub x_button: ButtonRole,
    pub y_button: ButtonRole,
    pub frame_blending: bool,
    /// Colour-grade preset, 0 (Full Color) to 6 (see `slot_gfx::colour_grade`).
    pub colour_grade: u8,
    /// How strongly the screen shader's mask shows: 50 or 100 (percent).
    pub shader_strength: u8,
    /// Stems of the carts the player starred.
    pub favorites: std::collections::BTreeSet<String>,
    /// On-screen controls: how opaque (percent of the default) and how big (percent).
    pub touch_opacity: u8,
    pub touch_size: u8,
    pub touch_haptics: bool,
    pub touch_mode: TouchMode,
    /// How strong the phone's motor runs, in percent: touch ticks and game rumble.
    pub vibration: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Shader {
    Off,
    Lcd3x,
    Grid,
    Dot,
    Simpletex,
}

impl Shader {
    pub const GBA: [Shader; 4] = [Shader::Off, Shader::Lcd3x, Shader::Grid, Shader::Dot];
    pub const GB: [Shader; 3] = [Shader::Off, Shader::Grid, Shader::Simpletex];

    pub fn name(self) -> &'static str {
        match self {
            Shader::Off => "off",
            Shader::Lcd3x => "lcd3x",
            Shader::Grid => "grid",
            Shader::Dot => "dot",
            Shader::Simpletex => "simpletex",
        }
    }

    pub fn parse(name: &str) -> Option<Shader> {
        [
            Shader::Off,
            Shader::Lcd3x,
            Shader::Grid,
            Shader::Dot,
            Shader::Simpletex,
        ]
        .into_iter()
        .find(|s| s.name() == name)
    }

    pub fn step(self, choices: &[Shader], right: bool) -> Shader {
        let n = choices.len();
        let at = choices.iter().position(|&s| s == self).unwrap_or(0);
        choices[if right {
            (at + 1) % n
        } else {
            (at + n - 1) % n
        }]
    }
}

impl Default for SlotState {
    fn default() -> Self {
        SlotState {
            cart: None,
            cart_platform: None,
            brightness: 5,
            blue_light: 0,
            volume: 60,
            volume_hp: 60,
            muted: false,
            muted_hp: false,
            clock_set: false,
            utc_offset_min: 0,
            rumble: true,
            ff_speed: FF_SPEED_DEFAULT,
            ff_sound: false,
            colour_correction: false,
            shader_gba: Shader::Grid,
            shader_gb: Shader::Simpletex,
            eject_save: true,
            turbo: true,
            rewind: true,
            gb_palettes: false,
            gb_palette: GbPalette::DEFAULT,
            x_button: ButtonRole::Turbo,
            y_button: ButtonRole::Turbo,
            frame_blending: true,
            colour_grade: 0,
            shader_strength: 100,
            favorites: Default::default(),
            touch_opacity: 100,
            touch_size: 100,
            touch_haptics: true,
            touch_mode: TouchMode::Auto,
            vibration: 70,
        }
    }
}

fn state_path(root: &Path) -> PathBuf {
    root.join("Config").join("slot.state")
}

pub fn read_slot_state(root: &Path) -> SlotState {
    std::fs::read(state_path(root))
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| parse(&s))
        .unwrap_or_default()
}

pub fn write_slot_state(root: &Path, s: &SlotState) -> std::io::Result<()> {
    let text = format!(
        "cart={}\ncart_platform={}\nbrightness={}\nblue_light={}\nvolume={}\nvolume_hp={}\nmuted={}\nmuted_hp={}\nclock_set={}\nutc_offset_min={}\nrumble={}\nff_speed={}\nff_sound={}\ncolour_correction={}\nshader_gba={}\nshader_gb={}\neject_save={}\nturbo={}\nrewind={}\ngb_palettes={}\ngb_palette={}\nx_button={}\ny_button={}\nframe_blending={}\ncolour_grade={}\nshader_strength={}\n",
        s.cart.as_deref().unwrap_or(""),
        s.cart_platform.map_or(String::new(), platform_key),
        s.brightness,
        s.blue_light,
        s.volume,
        s.volume_hp,
        s.muted as u8,
        s.muted_hp as u8,
        s.clock_set as u8,
        s.utc_offset_min,
        s.rumble as u8,
        s.ff_speed,
        s.ff_sound as u8,
        s.colour_correction as u8,
        s.shader_gba.name(),
        s.shader_gb.name(),
        s.eject_save as u8,
        s.turbo as u8,
        s.rewind as u8,
        s.gb_palettes as u8,
        s.gb_palette.core_name(),
        s.x_button.name(),
        s.y_button.name(),
        s.frame_blending as u8,
        s.colour_grade,
        s.shader_strength
    );
    // One line per stem so a stem carrying `=` or a space survives the round trip. Written last;
    // the reader tolerates the key's absence (a card that never starred anything has no line).
    let mut text = text;
    text.push_str(&format!(
        "touch_opacity={}\ntouch_size={}\ntouch_haptics={}\ntouch_mode={}\nvibration={}\n",
        s.touch_opacity, s.touch_size, s.touch_haptics as u8, s.touch_mode.name(), s.vibration
    ));
    for stem in &s.favorites {
        if !stem.contains('\n') {
            text.push_str(&format!("favorite={stem}\n"));
        }
    }
    atomic_write(&state_path(root), text.as_bytes())
}

fn parse(text: &str) -> Option<SlotState> {
    let mut cart = None;
    let mut cart_platform = None;
    let mut brightness = None;
    let mut blue_light = None;
    let mut volume = None;
    let mut volume_hp = None;
    let mut muted = None;
    let mut muted_hp = None;
    let mut clock_set = None;
    let mut utc_offset_min = None;
    let mut rumble = None;
    let mut ff_speed = None;
    let mut ff_sound = None;
    let mut colour_correction = None;
    let mut shader_gba = None;
    let mut shader_gb = None;
    let mut eject_save = None;
    let mut turbo = None;
    let mut rewind = None;
    let mut gb_palettes = None;
    let mut gb_palette = None;
    let mut x_button = None;
    let mut y_button = None;
    let mut frame_blending = None;
    let mut colour_grade = None;
    let mut shader_strength = None;
    let mut favorites = std::collections::BTreeSet::new();
    let mut touch_opacity = None;
    let mut touch_size = None;
    let mut touch_haptics = None;
    let mut touch_mode = None;
    let mut vibration = None;
    for line in text.lines().filter(|l| !l.is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "cart" => cart = Some(value.to_string()),
            "cart_platform" => cart_platform = platform_value(value),
            "brightness" => brightness = Some(level(value, BRIGHTNESS_MAX)?),
            "blue_light" => blue_light = Some(level(value, BLUE_LIGHT_MAX)?),
            "volume" => volume = Some(level(value, VOLUME_MAX)?),
            "volume_hp" => volume_hp = Some(level(value, VOLUME_MAX)?),
            "muted" => muted = Some(level(value, 1)? == 1),
            "muted_hp" => muted_hp = Some(level(value, 1)? == 1),
            "clock_set" => clock_set = Some(level(value, 1)? == 1),
            "utc_offset_min" => utc_offset_min = Some(offset(value)?),
            "rumble" => rumble = flag(value),
            "ff_speed" => ff_speed = ff_speed_value(value),
            "ff_sound" => ff_sound = flag(value),
            "colour_correction" => colour_correction = flag(value),
            "shader_gba" => shader_gba = Shader::parse(value).filter(|s| Shader::GBA.contains(s)),
            "eject_save" => eject_save = flag(value),
            "turbo" => turbo = flag(value),
            "rewind" => rewind = flag(value),
            "gb_palettes" => gb_palettes = flag(value),
            "gb_palette" => gb_palette = GbPalette::parse(value),
            "x_button" => x_button = ButtonRole::parse(value),
            "y_button" => y_button = ButtonRole::parse(value),
            "frame_blending" => frame_blending = flag(value),
            "touch_opacity" => {
                touch_opacity = value.parse::<u8>().ok().filter(|v| (20..=100).contains(v))
            }
            "touch_haptics" => touch_haptics = flag(value),
            "touch_mode" => touch_mode = TouchMode::parse(value),
            "vibration" => vibration = value.parse::<u8>().ok().filter(|v| (10..=150).contains(v)),
            "touch_size" => touch_size = value.parse::<u8>().ok().filter(|v| (60..=140).contains(v)),
            "favorite" => {
                favorites.insert(value.to_string());
            }
            "colour_grade" => colour_grade = value.parse::<u8>().ok().filter(|g| *g < 7),
            "shader_strength" => {
                shader_strength = value.parse::<u8>().ok().filter(|s| *s == 50 || *s == 100)
            }
            "shader_gb" => shader_gb = Shader::parse(value).filter(|s| Shader::GB.contains(s)),
            _ => {}
        }
    }
    let cart = cart?;
    let fallback = SlotState::default();
    Some(SlotState {
        cart: (!cart.is_empty()).then_some(cart),
        cart_platform,
        brightness: brightness?,
        blue_light: blue_light?,
        volume: volume?,
        volume_hp: volume_hp.or(volume)?,
        muted: muted?,
        muted_hp: muted_hp.or(muted)?,
        clock_set: clock_set?,
        utc_offset_min: utc_offset_min?,
        rumble: rumble.unwrap_or(fallback.rumble),
        ff_speed: ff_speed.unwrap_or(fallback.ff_speed),
        ff_sound: ff_sound.unwrap_or(fallback.ff_sound),
        colour_correction: colour_correction.unwrap_or(fallback.colour_correction),
        shader_gba: shader_gba.unwrap_or(fallback.shader_gba),
        shader_gb: shader_gb.unwrap_or(fallback.shader_gb),
        eject_save: eject_save.unwrap_or(fallback.eject_save),
        turbo: turbo.unwrap_or(fallback.turbo),
        rewind: rewind.unwrap_or(fallback.rewind),
        gb_palettes: gb_palettes.unwrap_or(fallback.gb_palettes),
        gb_palette: gb_palette.unwrap_or(fallback.gb_palette),
        x_button: x_button.unwrap_or(fallback.x_button),
        y_button: y_button.unwrap_or(fallback.y_button),
        frame_blending: frame_blending.unwrap_or(fallback.frame_blending),
        colour_grade: colour_grade.unwrap_or(fallback.colour_grade),
        shader_strength: shader_strength.unwrap_or(fallback.shader_strength),
        favorites,
        touch_opacity: touch_opacity.unwrap_or(fallback.touch_opacity),
        touch_size: touch_size.unwrap_or(fallback.touch_size),
        touch_haptics: touch_haptics.unwrap_or(fallback.touch_haptics),
        touch_mode: touch_mode.unwrap_or(fallback.touch_mode),
        vibration: vibration.unwrap_or(fallback.vibration),
    })
}

fn platform_key(platform: Platform) -> String {
    platform.dir_name().to_ascii_lowercase()
}

fn platform_value(value: &str) -> Option<Platform> {
    Platform::ALL
        .into_iter()
        .find(|p| value.eq_ignore_ascii_case(p.dir_name()))
}

fn offset(value: &str) -> Option<i16> {
    value
        .parse()
        .ok()
        .filter(|n| (UTC_OFFSET_MIN..=UTC_OFFSET_MAX).contains(n))
}

fn ff_speed_value(value: &str) -> Option<u8> {
    value.parse().ok().filter(|n| FF_SPEEDS.contains(n))
}

fn level(value: &str, max: u8) -> Option<u8> {
    value.parse().ok().filter(|n| *n <= max)
}

fn flag(value: &str) -> Option<bool> {
    level(value, 1).map(|n| n == 1)
}
