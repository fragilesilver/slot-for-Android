use android_activity::input::{InputEvent, KeyAction, Keycode, MotionAction};
use android_activity::{AndroidApp, InputStatus};
use android_activity::input::Axis;
use slot_store::TouchMode;
use slot_input::{Btn, InputSource, Millis, RawEvent};

use super::axes;
use crate::touch::{self, Zone};

pub struct AndroidInput {
    pending: Vec<RawEvent>,
    held: Vec<Btn>,
    touch_held: Vec<Btn>,
    presses: u32,
    menu_mode: bool,
    nav_at: Option<std::time::Instant>,
    hold: u8,
    taps: Vec<(f32, f32)>,
    window: (u32, u32),
    zones: Vec<Zone>,
    dim: f32,
    base_dim: f32,
    opacity: f32,
    size: f32,
    generation: u32,
    visible: bool,
    /// Directions held by a pad's hat or stick (as opposed to `DPAD_*` keys).
    axis_held: Vec<Btn>,
    mode: TouchMode,
}

fn key_to_btn(code: Keycode) -> Option<Btn> {
    Some(match code {
        Keycode::DpadUp => Btn::Up,
        Keycode::DpadDown => Btn::Down,
        Keycode::DpadLeft => Btn::Left,
        Keycode::DpadRight => Btn::Right,
        Keycode::ButtonA | Keycode::Z => Btn::A,
        Keycode::ButtonB | Keycode::X => Btn::B,
        Keycode::ButtonX | Keycode::C => Btn::X,
        Keycode::ButtonY | Keycode::V => Btn::Y,
        Keycode::ButtonL1 | Keycode::A => Btn::L1,
        Keycode::ButtonR1 | Keycode::S => Btn::R1,
        Keycode::ButtonL2 | Keycode::Q => Btn::L2,
        Keycode::ButtonR2 | Keycode::W => Btn::R2,
        Keycode::ButtonStart | Keycode::Enter => Btn::Start,
        Keycode::ButtonSelect | Keycode::ShiftRight => Btn::Select,
        Keycode::ButtonMode | Keycode::Tab | Keycode::Menu => Btn::Menu,
        _ => return None,
    })
}

impl AndroidInput {
    pub fn new() -> Self {
        AndroidInput {
            pending: Vec::new(),
            held: Vec::new(),
            touch_held: Vec::new(),
            presses: 0,
            menu_mode: false,
            nav_at: None,
            hold: 0,
            taps: Vec::new(),
            window: (0, 0),
            zones: Vec::new(),
            dim: 1.0,
            base_dim: 1.0,
            opacity: 1.0,
            size: 1.0,
            generation: 0,
            visible: true,
            axis_held: Vec::new(),
            mode: TouchMode::Auto,
        }
    }

    /// Tells the touch layer how big the window is; the controls are laid out from this.
    pub fn set_window(&mut self, window: (u32, u32)) {
        if window != self.window {
            self.window = window;
            self.base_dim = if touch::floats_over_game(window) { 1.7 } else { 1.0 };
            self.restyle();
        }
    }

    /// The player's opacity and size settings for the on-screen controls.
    pub fn set_style(&mut self, opacity: f32, size: f32) {
        if (opacity - self.opacity).abs() > 0.001 || (size - self.size).abs() > 0.001 {
            self.opacity = opacity;
            self.size = size;
            self.restyle();
        }
    }

    fn restyle(&mut self) {
        self.zones = touch::layout_sized(self.window, self.size);
        self.dim = self.base_dim * self.opacity;
        self.generation = self.generation.wrapping_add(1);
    }

    /// Bumps whenever the control layout changes (captions must be rebuilt).
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Whether the controls are drawn: always or never when pinned, and in Auto false while a
    /// physical pad or keyboard is in use (a touch brings them back).
    pub fn overlay_visible(&self) -> bool {
        match self.mode {
            TouchMode::Auto => self.visible,
            TouchMode::On => true,
            TouchMode::Off => false,
        }
    }

    /// The player's Touch Controls setting.
    pub fn set_mode(&mut self, mode: TouchMode) {
        self.mode = mode;
    }

    /// Idle opacity scale for the on-screen controls.
    pub fn dim(&self) -> f32 {
        self.dim
    }

    pub fn zones(&self) -> &[Zone] {
        &self.zones
    }

    /// Buttons currently pressed, from keys or fingers (to light the on-screen controls).
    pub fn pressed(&self) -> Vec<Btn> {
        let mut all = self.held.clone();
        all.extend_from_slice(&self.touch_held);
        all
    }

    /// While a menu is up the on-screen d-pad gives one direction at a time.
    pub fn set_menu_mode(&mut self, on: bool) {
        self.menu_mode = on;
    }

    fn touch(&mut self, points: &[(f32, f32)]) {
        let mut want: Vec<Btn> = Vec::new();
        let prev = self
            .touch_held
            .iter()
            .copied()
            .find(|b| matches!(b, Btn::Up | Btn::Down | Btn::Left | Btn::Right));
        for &(x, y) in points {
            for btn in touch::hit_with(&self.zones, x, y, self.menu_mode, prev) {
                if !want.contains(&btn) {
                    want.push(btn);
                }
            }
        }
        if self.menu_mode {
            // A thumb sitting on a sector edge can flicker; one new direction per beat is
            // plenty for walking a menu, and the rest wait until the next poll.
            const BEAT: std::time::Duration = std::time::Duration::from_millis(130);
            let now = std::time::Instant::now();
            let fresh = |b: &Btn| {
                matches!(b, Btn::Up | Btn::Down | Btn::Left | Btn::Right)
                    && !self.touch_held.contains(b)
            };
            if want.iter().any(fresh) {
                if self.nav_at.is_some_and(|t| now.duration_since(t) < BEAT) {
                    let held = self.touch_held.clone();
                    want.retain(|b| !matches!(b, Btn::Up | Btn::Down | Btn::Left | Btn::Right) || held.contains(b));
                } else {
                    self.nav_at = Some(now);
                }
            }
        }
        for &btn in &want {
            if !self.touch_held.contains(&btn) {
                crate::latency::read();
                self.presses += 1;
                self.pending.push(RawEvent::Down(btn));
            }
        }
        for &btn in &self.touch_held {
            if !want.contains(&btn) {
                self.pending.push(RawEvent::Up(btn));
            }
        }
        self.touch_held = want;
    }

    /// Presses and releases the D-PAD directions a pad's axes call for.
    fn set_axis_dirs(&mut self, want: &[Btn]) {
        for &btn in want {
            if !self.axis_held.contains(&btn) {
                self.axis_held.push(btn);
                if !self.held.contains(&btn) {
                    self.held.push(btn);
                    crate::latency::read();
                    self.pending.push(RawEvent::Down(btn));
                }
            }
        }
        let gone: Vec<Btn> = self.axis_held.iter().copied().filter(|b| !want.contains(b)).collect();
        for btn in gone {
            self.axis_held.retain(|b| *b != btn);
            self.held.retain(|b| *b != btn);
            self.pending.push(RawEvent::Up(btn));
        }
    }

    /// Drains the activity's queued key events into slot's raw button stream.
    pub fn pump(&mut self, app: &AndroidApp) {
        let Ok(mut events) = app.input_events_iter() else {
            return;
        };
        loop {
            let more = events.next(|event| {
                if let InputEvent::MotionEvent(motion) = event {
                    // A pad's hat or stick is not a finger: read it as a D-PAD and leave the
                    // touch controls (and their visibility) alone.
                    if u32::from(motion.source()) & axes::SOURCE_CLASS_JOYSTICK != 0 {
                        let p = motion.pointer_at_index(0);
                        let want = axes::dirs(
                            p.axis_value(Axis::HatX),
                            p.axis_value(Axis::HatY),
                            p.axis_value(Axis::X),
                            p.axis_value(Axis::Y),
                        );
                        if !want.is_empty() {
                            self.visible = false;
                        }
                        self.set_axis_dirs(&want);
                        return InputStatus::Handled;
                    }
                    let lifted = match motion.action() {
                        MotionAction::Up | MotionAction::PointerUp => Some(motion.pointer_index()),
                        _ => None,
                    };
                    if matches!(motion.action(), MotionAction::Down | MotionAction::PointerDown) {
                        let p = motion.pointer_at_index(motion.pointer_index());
                        // A press is a tap on the screen's content only when it lands on the
                        // picture area and not on one of the on-screen buttons; a thumb on the
                        // d-pad must not also poke the menu row at the same height.
                        let on_button = !touch::hit(&self.zones, p.x(), p.y()).is_empty();
                        if let Some(at) = crate::letter_bar::to_output(self.window, p.x(), p.y()) {
                            let inside = at.0 >= 0.0
                                && at.1 >= 0.0
                                && at.0 < slot_gfx::OUT_W as f32
                                && at.1 < slot_gfx::OUT_H as f32;
                            if inside && !on_button {
                                self.taps.push(at);
                            }
                        }
                    }
                    let cancel = matches!(motion.action(), MotionAction::Cancel);
                    let points: Vec<(f32, f32)> = motion
                        .pointers()
                        .filter(|p| !cancel && Some(p.pointer_index()) != lifted)
                        .map(|p| (p.x(), p.y()))
                        .collect();
                    self.visible = true;
                    self.touch(&points);
                    return InputStatus::Handled;
                }
                let InputEvent::KeyEvent(key) = event else {
                    return InputStatus::Unhandled;
                };
                // The phone's Back button / back swipe acts as a Menu tap (opens the pause
                // menu, closes it again). Swallowing both edges also stops Android finishing
                // the activity.
                if key.key_code() == Keycode::Back {
                    if key.action() == KeyAction::Down && key.repeat_count() == 0 {
                        self.pending.push(RawEvent::Down(Btn::Menu));
                        self.pending.push(RawEvent::Up(Btn::Menu));
                    }
                    return InputStatus::Handled;
                }
                let Some(btn) = key_to_btn(key.key_code()) else {
                    return InputStatus::Unhandled;
                };
                self.visible = false;
                match key.action() {
                    KeyAction::Down if !self.held.contains(&btn) => {
                        self.held.push(btn);
                        crate::latency::read();
                        self.pending.push(RawEvent::Down(btn));
                    }
                    KeyAction::Up => {
                        self.held.retain(|b| *b != btn);
                        self.pending.push(RawEvent::Up(btn));
                    }
                    _ => {}
                }
                InputStatus::Handled
            });
            if !more {
                break;
            }
        }
    }

    /// Touch-downs since the last call, in the 720x480 output space.
    /// How many touch buttons went down since the last call.
    pub fn take_presses(&mut self) -> u32 {
        std::mem::take(&mut self.presses)
    }

    pub fn take_taps(&mut self) -> Vec<(f32, f32)> {
        std::mem::take(&mut self.taps)
    }

    /// Releases everything (window lost focus or went away) so no button stays stuck.
    pub fn release_all(&mut self) {
        self.axis_held.clear();
        for btn in std::mem::take(&mut self.held) {
            self.pending.push(RawEvent::Up(btn));
        }
        for btn in std::mem::take(&mut self.touch_held) {
            self.pending.push(RawEvent::Up(btn));
        }
    }
}

impl Default for AndroidInput {
    fn default() -> Self {
        AndroidInput::new()
    }
}

impl InputSource for AndroidInput {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        // A very short tap delivers Down and Up in the same batch, and the emulator would
        // never see the button held. Release is held back so the press lasts a couple of polls.
        if self.hold > 0 {
            self.hold -= 1;
            return Vec::new();
        }
        let mut downs: Vec<Btn> = Vec::new();
        let mut split = None;
        for (i, event) in self.pending.iter().enumerate() {
            match event {
                RawEvent::Down(b) => downs.push(*b),
                RawEvent::Up(b) if downs.contains(b) => {
                    split = Some(i);
                    break;
                }
                _ => {}
            }
        }
        match split {
            Some(i) => {
                self.hold = 1;
                self.pending.drain(..i).collect()
            }
            None => std::mem::take(&mut self.pending),
        }
    }
}
