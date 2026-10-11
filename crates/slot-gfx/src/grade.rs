const WARMEST: [f32; 3] = [1.0, 0.82, 0.62];

pub const BLUE_LIGHT_MAX: u8 = 9;

pub fn blue_light_gain(step: u8) -> [f32; 3] {
    let t = step.min(BLUE_LIGHT_MAX) as f32 / BLUE_LIGHT_MAX as f32;
    let mut gain = [1.0f32; 3];
    for (g, warm) in gain.iter_mut().zip(WARMEST) {
        *g = 1.0 + (warm - 1.0) * t;
    }
    gain
}

/// Number of colour-grade presets `colour_grade` knows (0 is "Full Color", an identity).
pub const COLOUR_GRADES: u8 = 7;

const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];
const GRADE_GAMMA: f32 = 2.2;

fn linear(rgb: [u8; 3]) -> [f32; 3] {
    rgb.map(|v| (v as f32 / 255.0).powf(GRADE_GAMMA))
}

fn tint(rgb: [u8; 3]) -> [f32; 9] {
    let target = linear(rgb);
    let mut m = [0.0; 9];
    for row in 0..3 {
        for col in 0..3 {
            m[row * 3 + col] = target[row] * LUMA[col];
        }
    }
    m
}

/// `(row-major 3x3 matrix, gamma)` applied in linear light; `None` for the untouched picture.
/// 0 Full Color, 1 Half Color, 2 No Color, 3 DMG green, 4 Amber, 5 Ice blue, 6 Pink.
pub fn colour_grade(index: u8) -> Option<([f32; 9], f32)> {
    let mono = [
        LUMA[0], LUMA[1], LUMA[2], LUMA[0], LUMA[1], LUMA[2], LUMA[0], LUMA[1], LUMA[2],
    ];
    match index {
        1 => {
            let mut m = [0.0; 9];
            for i in 0..9 {
                let identity = if i % 4 == 0 { 1.0 } else { 0.0 };
                m[i] = 0.5 * identity + 0.5 * mono[i];
            }
            Some((m, GRADE_GAMMA))
        }
        2 => Some((mono, GRADE_GAMMA)),
        3 => Some((tint([155, 188, 15]), GRADE_GAMMA)),
        4 => Some((tint([240, 165, 60]), GRADE_GAMMA)),
        5 => Some((tint([120, 170, 215]), GRADE_GAMMA)),
        6 => Some((tint([240, 130, 185]), GRADE_GAMMA)),
        _ => None,
    }
}

#[cfg(test)]
mod grade_tests {
    use super::*;

    #[test]
    fn full_colour_is_untouched_and_the_rest_keep_white_white() {
        assert!(colour_grade(0).is_none());
        for i in 1..COLOUR_GRADES {
            let (m, _) = colour_grade(i).unwrap();
            let white: Vec<f32> = (0..3).map(|r| m[r * 3] + m[r * 3 + 1] + m[r * 3 + 2]).collect();
            if i <= 2 {
                assert!(white.iter().all(|w| (w - 1.0).abs() < 1e-4), "{i}: {white:?}");
            } else {
                assert!(white.iter().all(|w| *w <= 1.0 + 1e-4), "{i}: {white:?}");
            }
        }
    }
}
