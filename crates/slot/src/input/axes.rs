//! A controller's D-PAD or left stick, read from motion axes, as the four directions.
//!
//! Many pads report the D-PAD as the hat axes of a joystick motion event, not as `DPAD_*`
//! keys. The stick is folded in too, with a dead zone so a resting thumb sends nothing.
use slot_input::Btn;

/// Past this deflection a stick counts as a direction (the hat is always -1, 0 or 1).
const DEAD_ZONE: f32 = 0.5;

/// Bit in an input source that marks a joystick or gamepad (`SOURCE_CLASS_JOYSTICK`); a
/// touchscreen is a pointer source and never has it.
pub const SOURCE_CLASS_JOYSTICK: u32 = 0x10;

/// The directions held for a hat (`hat_x`, `hat_y`) and a stick (`x`, `y`); the hat wins on an
/// axis it is using. Up and Left are negative, as on the screen.
pub fn dirs(hat_x: f32, hat_y: f32, x: f32, y: f32) -> Vec<Btn> {
    let pick = |hat: f32, stick: f32| {
        if hat.abs() > 0.5 {
            hat
        } else if stick.abs() > DEAD_ZONE {
            stick
        } else {
            0.0
        }
    };
    let (h, v) = (pick(hat_x, x), pick(hat_y, y));
    let mut out = Vec::with_capacity(2);
    if v < 0.0 {
        out.push(Btn::Up);
    } else if v > 0.0 {
        out.push(Btn::Down);
    }
    if h < 0.0 {
        out.push(Btn::Left);
    } else if h > 0.0 {
        out.push(Btn::Right);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resting_pad_sends_nothing() {
        assert!(dirs(0.0, 0.0, 0.0, 0.0).is_empty());
        assert!(dirs(0.0, 0.0, 0.2, -0.3).is_empty());
    }

    #[test]
    fn the_hat_gives_the_four_directions() {
        assert_eq!(dirs(0.0, -1.0, 0.0, 0.0), vec![Btn::Up]);
        assert_eq!(dirs(0.0, 1.0, 0.0, 0.0), vec![Btn::Down]);
        assert_eq!(dirs(-1.0, 0.0, 0.0, 0.0), vec![Btn::Left]);
        assert_eq!(dirs(1.0, 0.0, 0.0, 0.0), vec![Btn::Right]);
    }

    #[test]
    fn a_diagonal_is_two_directions() {
        assert_eq!(dirs(1.0, -1.0, 0.0, 0.0), vec![Btn::Up, Btn::Right]);
    }

    #[test]
    fn a_stick_past_the_dead_zone_counts() {
        assert_eq!(dirs(0.0, 0.0, -0.9, 0.0), vec![Btn::Left]);
        assert_eq!(dirs(0.0, 0.0, 0.0, 0.9), vec![Btn::Down]);
    }

    #[test]
    fn the_hat_wins_over_the_stick_on_its_axis() {
        assert_eq!(dirs(1.0, 0.0, -0.9, 0.0), vec![Btn::Right]);
    }
}
