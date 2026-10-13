//! On-screen gamepad: layout, hit-testing and drawing shapes. Pure logic, no Android types.
use slot_gfx::Shape;
use slot_input::Btn;

const GAME_W: f32 = 720.0;
const GAME_H: f32 = 480.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Zone {
    Dpad { c: (f32, f32), r: f32 },
    Circle { btn: Btn, c: (f32, f32), r: f32 },
    Pill { btn: Btn, rect: [f32; 4] },
}

fn pill(btn: Btn, cx: f32, cy: f32, w: f32, h: f32) -> Zone {
    Zone::Pill {
        btn,
        rect: [cx - w / 2.0, cy - h / 2.0, w, h],
    }
}

/// True when the controls float over the game picture: a near-square window (an unfolded
/// phone) has no room for side bars or a roomy band underneath.
pub fn floats_over_game(window: (u32, u32)) -> bool {
    if slot_gfx::is_portrait(window) {
        return false;
    }
    let (w, h) = (window.0 as f32, window.1 as f32);
    let scale = (w / GAME_W).min(h / GAME_H);
    let (gw, gh) = (GAME_W * scale, GAME_H * scale);
    let bar = (w - gw) / 2.0;
    let below = (h - gh) / 2.0;
    bar < 0.18 * h && below < 0.45 * w
}

/// The shoulder row on one side: the outer half is L (or R), the inner half L2 (or R2), so
/// rewind and fast forward are a thumb away without crowding the screen.
fn shoulders(zones: &mut Vec<Zone>, left: bool, cx: f32, cy: f32, w: f32, h: f32) {
    let half = w / 2.0 * 0.94;
    let (outer, inner) = (cx + if left { -w / 4.0 } else { w / 4.0 }, cx + if left { w / 4.0 } else { -w / 4.0 });
    let (a, b) = if left { (Btn::L1, Btn::L2) } else { (Btn::R1, Btn::R2) };
    zones.push(pill(a, outer, cy, half, h));
    zones.push(pill(b, inner, cy, half, h));
}

/// X over A / Y / B diamond (same shape as the d-pad): X top, Y left, A right, B bottom.
fn face_diamond(zones: &mut Vec<Zone>, c: (f32, f32), r: f32) {
    let d = r * 0.62;
    let br = r * 0.42;
    zones.push(Zone::Circle { btn: Btn::X, c: (c.0, c.1 - d), r: br });
    zones.push(Zone::Circle { btn: Btn::Y, c: (c.0 - d, c.1), r: br });
    zones.push(Zone::Circle { btn: Btn::A, c: (c.0 + d, c.1), r: br });
    zones.push(Zone::Circle { btn: Btn::B, c: (c.0, c.1 + d), r: br });
}

/// Lays the controls out around the (aspect-fitted) game picture: in the side bars when the
/// window is wide, in the band under the picture when it is tall.
pub fn layout(window: (u32, u32)) -> Vec<Zone> {
    layout_sized(window, 1.0)
}

/// `layout` with every control grown or shrunk by `size` about its own centre.
pub fn layout_sized(window: (u32, u32), size: f32) -> Vec<Zone> {
    let (w, h) = (window.0 as f32, window.1 as f32);
    let scale = (w / GAME_W).min(h / GAME_H);
    let (gw, gh) = (GAME_W * scale, GAME_H * scale);
    let bar = (w - gw) / 2.0;
    let mut zones = Vec::new();
    if bar >= 0.18 * h {
        let r = (bar * 0.40).min(h * 0.26) * size;
        let (lx, rx) = (bar / 2.0, w - bar / 2.0);
        let pw = (bar * 0.8).min(h * 0.4);
        shoulders(&mut zones, true, lx, h * 0.16, pw * size, (h * 0.11) * size);
        shoulders(&mut zones, false, rx, h * 0.16, pw * size, (h * 0.11) * size);
        // Clear of the strip along the top edge that the system keeps for its own swipe.
        zones.push(pill(Btn::Menu, w * 0.5, h * 0.085, (pw * 0.9) * size, (h * 0.07) * size));
        zones.push(Zone::Dpad { c: (lx, h * 0.55), r });
        face_diamond(&mut zones, (rx, h * 0.55), r);
        zones.push(pill(Btn::Select, lx, h * 0.88, (pw * 0.7) * size, (h * 0.08) * size));
        zones.push(pill(Btn::Start, rx, h * 0.88, (pw * 0.7) * size, (h * 0.08) * size));
    } else if floats_over_game(window) {
        // Big controls over the picture's lower corners and top edge; drawn translucent.
        let (gy0, gy1) = ((h - gh) / 2.0, (h + gh) / 2.0);
        let r = w * 0.13 * size;
        let anchor = gy1 - r * 1.35;
        let pw = w * 0.2;
        shoulders(&mut zones, true, w * 0.13, gy0 + h * 0.05, pw * size, (h * 0.055) * size);
        shoulders(&mut zones, false, w * 0.87, gy0 + h * 0.05, pw * size, (h * 0.055) * size);
        zones.push(pill(Btn::Menu, w * 0.5, gy0 + h * 0.05, (pw * 0.8) * size, (h * 0.045) * size));
        zones.push(Zone::Dpad { c: (w * 0.16, anchor), r });
        face_diamond(&mut zones, (w * 0.84, anchor), r);
        zones.push(pill(Btn::Select, w * 0.40, gy1 - h * 0.035, (pw * 0.8) * size, (h * 0.045) * size));
        zones.push(pill(Btn::Start, w * 0.60, gy1 - h * 0.035, (pw * 0.8) * size, (h * 0.045) * size));
    } else {
        // Held upright the picture hangs under the cutout strip; elsewhere it is centred.
        let top = if slot_gfx::is_portrait(window) {
            let (_, y, _, ph) = slot_gfx::portrait_rect(window);
            (y + ph) as f32
        } else {
            (h - gh) / 2.0 + gh
        };
        let free = (h - top).max(h * 0.2);
        let r = (w * 0.17).min(free * 0.24) * size;
        let pw = w * 0.32;
        shoulders(&mut zones, true, w * 0.2, top + free * 0.08, pw * size, (free * 0.07) * size);
        shoulders(&mut zones, false, w * 0.8, top + free * 0.08, pw * size, (free * 0.07) * size);
        zones.push(pill(Btn::Menu, w * 0.5, top + free * 0.08, (pw * 0.7) * size, (free * 0.06) * size));
        zones.push(Zone::Dpad { c: (w * 0.25, top + free * 0.40), r });
        face_diamond(&mut zones, (w * 0.75, top + free * 0.40), r);
        zones.push(pill(Btn::Select, w * 0.36, top + free * 0.86, (pw * 0.7) * size, (free * 0.06) * size));
        zones.push(pill(Btn::Start, w * 0.64, top + free * 0.86, (pw * 0.7) * size, (free * 0.06) * size));
    }
    zones
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// Every button a finger at `(x, y)` presses (the d-pad can give two for a diagonal).
pub fn hit(zones: &[Zone], x: f32, y: f32) -> Vec<Btn> {
    hit_with(zones, x, y, false, None)
}

/// Which one of the four directions a finger at the pad-relative unit vector `(nx, ny)` means.
/// `prev` is the direction already held; it is kept until the finger is clearly on the other
/// axis, so a thumb resting near 45 degrees does not flicker between two directions.
fn cardinal(nx: f32, ny: f32, prev: Option<Btn>) -> Btn {
    const STICK: f32 = 1.3;
    let vertical = match prev {
        Some(Btn::Up | Btn::Down) => ny.abs() * STICK >= nx.abs(),
        Some(Btn::Left | Btn::Right) => ny.abs() > nx.abs() * STICK,
        _ => ny.abs() >= nx.abs(),
    };
    match (vertical, ny < 0.0, nx < 0.0) {
        (true, true, _) => Btn::Up,
        (true, false, _) => Btn::Down,
        (false, _, true) => Btn::Left,
        (false, _, false) => Btn::Right,
    }
}

/// `hit`, with `menu` narrowing the d-pad to one direction at a time (menus never want the
/// diagonal that an off-axis thumb makes), `prev` being the direction held right now.
pub fn hit_with(zones: &[Zone], x: f32, y: f32, menu: bool, prev: Option<Btn>) -> Vec<Btn> {
    let mut out = Vec::new();
    // A button under the finger wins over the d-pad's generous catch area, so tapping Menu or
    // a shoulder button near the pad never also presses Up.
    let on_button = zones.iter().any(|z| match *z {
        Zone::Circle { c, r, .. } => dist(c, (x, y)) <= r * 1.2,
        Zone::Pill { rect, .. } => {
            let (px, py) = (rect[2] * 0.1, rect[3] * 0.3);
            x >= rect[0] - px
                && x <= rect[0] + rect[2] + px
                && y >= rect[1] - py
                && y <= rect[1] + rect[3] + py
        }
        Zone::Dpad { .. } => false,
    });
    for zone in zones {
        if on_button && matches!(zone, Zone::Dpad { .. }) {
            continue;
        }
        match *zone {
            Zone::Dpad { c, r } => {
                let d = dist(c, (x, y));
                if d <= r * 1.4 && d >= r * 0.22 {
                    let (nx, ny) = ((x - c.0) / d, (y - c.1) / d);
                    if menu {
                        out.push(cardinal(nx, ny, prev));
                        continue;
                    }
                    for (on, btn) in [
                        (ny < -0.38, Btn::Up),
                        (ny > 0.38, Btn::Down),
                        (nx < -0.38, Btn::Left),
                        (nx > 0.38, Btn::Right),
                    ] {
                        if on {
                            out.push(btn);
                        }
                    }
                }
            }
            Zone::Circle { btn, c, r } => {
                if dist(c, (x, y)) <= r * 1.2 {
                    out.push(btn);
                }
            }
            Zone::Pill { btn, rect } => {
                let (px, py) = (rect[2] * 0.1, rect[3] * 0.3);
                if x >= rect[0] - px
                    && x <= rect[0] + rect[2] + px
                    && y >= rect[1] - py
                    && y <= rect[1] + rect[3] + py
                {
                    out.push(btn);
                }
            }
        }
    }
    out
}

fn tint(btn: Btn, held: bool, dim: f32) -> [f32; 4] {
    let a = if held { 0.62 } else { 0.28 * dim };
    match btn {
        Btn::A => [0.95, 0.70, 0.15, a],
        Btn::B => [0.90, 0.32, 0.28, a],
        Btn::X => [0.35, 0.60, 0.95, a],
        Btn::Y => [0.40, 0.80, 0.45, a],
        _ => [0.85, 0.85, 0.90, a],
    }
}

/// `dim` scales the idle opacity (use < 1.0 when the controls float over the game).
pub fn shapes(zones: &[Zone], held: &[Btn], dim: f32) -> Vec<Shape> {
    let is = |b: Btn| held.contains(&b);
    let mut out = Vec::new();
    for zone in zones {
        match *zone {
            Zone::Dpad { c, r } => {
                let arm = r * 0.62;
                out.push(Shape {
                    rect: [c.0 - r, c.1 - r, r * 2.0, r * 2.0],
                    round: true,
                    rgba: [0.85, 0.85, 0.90, 0.12 * dim],
                    label: None,
                });
                for (btn, dx, dy) in [
                    (Btn::Up, 0.0, -1.0),
                    (Btn::Down, 0.0, 1.0),
                    (Btn::Left, -1.0, 0.0),
                    (Btn::Right, 1.0, 0.0),
                ] {
                    let (cx, cy) = (c.0 + dx * r * 0.55, c.1 + dy * r * 0.55);
                    out.push(Shape {
                        rect: [cx - arm / 2.0, cy - arm / 2.0, arm, arm],
                        round: true,
                        rgba: tint(btn, is(btn), dim),
                        label: None,
                    });
                }
            }
            Zone::Circle { btn, c, r } => out.push(Shape {
                rect: [c.0 - r, c.1 - r, r * 2.0, r * 2.0],
                round: true,
                rgba: tint(btn, is(btn), dim),
                label: None,
            }),
            Zone::Pill { btn, rect } => out.push(Shape {
                rect,
                round: false,
                rgba: tint(btn, is(btn), dim),
                label: None,
            }),
        }
    }
    out
}

/// The caption a control carries and the box it is centred in.
pub fn caption(zone: &Zone) -> Option<(&'static str, [f32; 4])> {
    match *zone {
        Zone::Dpad { .. } => None,
        Zone::Circle { btn, c, r } => {
            let text = match btn {
                Btn::A => "A",
                Btn::B => "B",
                Btn::X => "X",
                Btn::Y => "Y",
                _ => return None,
            };
            Some((text, [c.0 - r, c.1 - r, r * 2.0, r * 2.0]))
        }
        Zone::Pill { btn, rect } => {
            let text = match btn {
                Btn::L1 => "L",
                Btn::R1 => "R",
                Btn::L2 => "L2",
                Btn::R2 => "R2",
                Btn::Start => "START",
                Btn::Select => "SELECT",
                Btn::Menu => "MENU",
                _ => return None,
            };
            Some((text, rect))
        }
    }
}

/// White text with its coverage as alpha, centred in a `w` x `h` bitmap.
pub fn caption_bitmap(text: &str, w: u32, h: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let Some(font) = slot_ui::text::label_font() else {
        return rgba;
    };
    let size = (h as f32 * 0.5).max(8.0);
    let layout = slot_ui::text::fit(font, text, w as f32 * 0.9, 1, size, 6.0);
    slot_ui::text::draw_centred(&mut rgba, w, h, &layout, [255, 255, 255]);
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    fn centre(z: &Zone) -> (f32, f32) {
        match *z {
            Zone::Dpad { c, .. } | Zone::Circle { c, .. } => c,
            Zone::Pill { rect, .. } => (rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0),
        }
    }

    #[test]
    fn every_button_is_reachable_at_its_own_centre() {
        for window in [(2520, 1080), (1080, 2520), (1968, 2184), (2184, 1968)] {
            let zones = layout(window);
            for z in &zones {
                let (x, y) = centre(z);
                if matches!(z, Zone::Dpad { .. }) {
                    continue;
                }
                let want = match *z {
                    Zone::Circle { btn, .. } | Zone::Pill { btn, .. } => btn,
                    _ => unreachable!(),
                };
                assert!(hit(&zones, x, y).contains(&want), "{window:?} {want:?}");
            }
        }
    }

    #[test]
    fn dpad_gives_diagonals_and_a_dead_centre() {
        let zones = layout((2520, 1080));
        let Some(Zone::Dpad { c, r }) = zones.iter().copied().find(|z| matches!(z, Zone::Dpad { .. }))
        else {
            panic!("no dpad")
        };
        assert!(hit(&zones, c.0, c.1).is_empty());
        assert_eq!(hit(&zones, c.0, c.1 - r * 0.8), vec![Btn::Up]);
        let both = hit(&zones, c.0 + r * 0.7, c.1 - r * 0.7);
        assert!(both.contains(&Btn::Up) && both.contains(&Btn::Right));
    }

    #[test]
    fn unfolded_screen_floats_the_controls_inside_the_window() {
        let window = (2184, 1968);
        assert!(floats_over_game(window));
        assert!(!floats_over_game((2520, 1080)));
        assert!(!floats_over_game((1080, 2520)));
        let zones = layout(window);
        for s in shapes(&zones, &[], 0.5) {
            let [x, y, w, h] = s.rect;
            assert!(x >= 0.0 && y >= 0.0, "{s:?} starts off screen");
            assert!(x + w <= window.0 as f32 && y + h <= window.1 as f32, "{s:?} ends off screen");
        }
        for z in &zones {
            let (x, y) = centre(z);
            let want = match *z {
                Zone::Dpad { .. } => continue,
                Zone::Circle { btn, .. } | Zone::Pill { btn, .. } => btn,
            };
            assert!(hit(&zones, x, y).contains(&want), "{want:?} unreachable when floating");
        }
    }

    #[test]
    fn held_upright_the_picture_clears_the_cutout_and_the_controls_sit_under_it() {
        for window in [(1080, 2520), (720, 1520), (1440, 3040)] {
            assert!(slot_gfx::is_portrait(window), "{window:?}");
            let (px, py, pw, ph) = slot_gfx::portrait_rect(window);
            // Full width, and a strip kept clear at the top for the camera cutout.
            assert_eq!((px, pw), (0, window.0 as i32));
            assert!(py as f32 >= window.1 as f32 * 0.04, "{window:?} no room for the cutout");
            // The window is drawn where the touch layer thinks the picture is.
            let (_, gl_y, _, gl_h) = slot_gfx::blit_rect(window, 0.0);
            assert_eq!(window.1 as i32 - gl_y - gl_h, py);
            assert_eq!(gl_h, ph);
            let zones = layout(window);
            assert!(!floats_over_game(window));
            for s in shapes(&zones, &[], 1.0) {
                let [x, y, w, h] = s.rect;
                assert!(x >= 0.0 && x + w <= window.0 as f32, "{window:?} {s:?} off the sides");
                assert!(y + h <= window.1 as f32, "{window:?} {s:?} off the bottom");
                assert!(y >= (py + ph) as f32 - 1.0, "{window:?} {s:?} over the picture");
            }
        }
        // A near-square window is not portrait: the picture stays centred.
        assert!(!slot_gfx::is_portrait((1968, 2184)));
    }

    #[test]
    fn controls_stay_off_the_game_picture() {
        let (w, h) = (2520.0, 1080.0);
        let gw = GAME_W * (w / GAME_W).min(h / GAME_H);
        let bar = (w - gw) / 2.0;
        let zones = layout((2520, 1080));
        for (zone, s) in zones.iter().zip(shapes(&zones, &[], 1.0)) {
            // The Menu button sits at the top centre, over the picture's top edge, on purpose.
            if matches!(zone, Zone::Pill { btn: Btn::Menu, .. }) {
                continue;
            }
            let (l, r) = (s.rect[0], s.rect[0] + s.rect[2]);
            assert!(r <= bar + 1.0 || l >= w - bar - 1.0, "{s:?} overlaps the picture");
        }
    }

    #[test]
    fn in_a_menu_the_pad_gives_one_direction_and_does_not_flicker() {
        let zones = layout((2520, 1080));
        let (c, r) = zones
            .iter()
            .find_map(|z| match z {
                Zone::Dpad { c, r } => Some((*c, *r)),
                _ => None,
            })
            .unwrap();
        // A thumb on the up arm, drifting left of straight up, is still just Up.
        for dx in [-0.5, -0.3, 0.0, 0.3, 0.5] {
            let got = hit_with(&zones, c.0 + dx * r * 0.5, c.1 - r * 0.8, true, None);
            assert_eq!(got, vec![Btn::Up], "dx {dx}");
        }
        // Held Up stays Up a little past the 45 degree line, and gives way beyond it.
        let near = (c.0 - r * 0.7, c.1 - r * 0.6);
        assert_eq!(hit_with(&zones, near.0, near.1, true, Some(Btn::Up)), vec![Btn::Up]);
        assert_eq!(hit_with(&zones, near.0, near.1, true, Some(Btn::Left)), vec![Btn::Left]);
        assert_eq!(hit_with(&zones, c.0 - r, c.1 - r * 0.1, true, Some(Btn::Up)), vec![Btn::Left]);
    }

    #[test]
    fn menu_is_at_the_top_centre_and_never_also_presses_up() {
        for window in [(2520, 1080), (1080, 2520), (2200, 2000)] {
            let zones = layout(window);
            let menu = zones
                .iter()
                .find_map(|z| match z {
                    Zone::Pill { btn: Btn::Menu, rect } => Some(*rect),
                    _ => None,
                })
                .unwrap();
            let (cx, cy) = (menu[0] + menu[2] / 2.0, menu[1] + menu[3] / 2.0);
            assert!((cx - window.0 as f32 / 2.0).abs() < 1.0, "{window:?}");
            assert_eq!(hit(&zones, cx, cy), vec![Btn::Menu], "{window:?}");
            // Every pill's centre presses only itself, whatever the d-pad's reach.
            for z in &zones {
                if let Zone::Pill { btn, rect } = z {
                    let (x, y) = (rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0);
                    assert_eq!(hit(&zones, x, y), vec![*btn], "{window:?} {btn:?}");
                }
            }
        }
    }
}
