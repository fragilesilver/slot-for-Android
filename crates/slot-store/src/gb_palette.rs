const NAMES: [&str; 48] = [
    "Grayscale",
    "DMG Green",
    "GB Pocket",
    "GB Light",
    "GBC Brown ↑",
    "GBC Red ↑A",
    "GBC Dark Brown ↑B",
    "GBC Pale Yellow ↓",
    "GBC Orange ↓A",
    "GBC Yellow ↓B",
    "GBC Blue ←",
    "GBC Dark Blue ←A",
    "GBC Gray ←B",
    "GBC Green →",
    "GBC Dark Green →A",
    "GBC Reverse →B",
    "SGB 1-A",
    "SGB 1-B",
    "SGB 1-C",
    "SGB 1-D",
    "SGB 1-E",
    "SGB 1-F",
    "SGB 1-G",
    "SGB 1-H",
    "SGB 2-A",
    "SGB 2-B",
    "SGB 2-C",
    "SGB 2-D",
    "SGB 2-E",
    "SGB 2-F",
    "SGB 2-G",
    "SGB 2-H",
    "SGB 3-A",
    "SGB 3-B",
    "SGB 3-C",
    "SGB 3-D",
    "SGB 3-E",
    "SGB 3-F",
    "SGB 3-G",
    "SGB 3-H",
    "SGB 4-A",
    "SGB 4-B",
    "SGB 4-C",
    "SGB 4-D",
    "SGB 4-E",
    "SGB 4-F",
    "SGB 4-G",
    "SGB 4-H",
];

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct GbPalette(u8);

impl GbPalette {
    pub const COUNT: usize = NAMES.len();
    pub const DEFAULT: GbPalette = GbPalette(1);

    pub fn all() -> impl Iterator<Item = GbPalette> {
        (0..Self::COUNT as u8).map(GbPalette)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn core_name(self) -> &'static str {
        NAMES[self.index()]
    }

    pub fn label(self) -> &'static str {
        let name = self.core_name();
        match name.find(['↑', '↓', '←', '→']) {
            Some(at) => name[..at].trim_end(),
            None => name,
        }
    }

    pub fn next(self) -> GbPalette {
        GbPalette(((self.index() + 1) % Self::COUNT) as u8)
    }

    pub fn prev(self) -> GbPalette {
        GbPalette(((self.index() + Self::COUNT - 1) % Self::COUNT) as u8)
    }

    /// One step round the ring Off, then every palette, as the menu row has it. `on` is whether
    /// palettes are in use and `at` the palette they are on; Off keeps `at` as it was.
    pub fn step(on: bool, at: GbPalette, right: bool) -> (bool, GbPalette) {
        let ring = Self::COUNT + 1;
        let here = if on { at.index() + 1 } else { 0 };
        let to = (here + if right { 1 } else { ring - 1 }) % ring;
        match to {
            0 => (false, at),
            n => (true, GbPalette((n - 1) as u8)),
        }
    }

    pub fn parse(name: &str) -> Option<GbPalette> {
        Self::all().find(|p| p.core_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_row_walks_off_then_every_palette_then_off_again() {
        let (mut on, mut at) = (false, GbPalette::DEFAULT);
        let mut seen = Vec::new();
        for _ in 0..=GbPalette::COUNT {
            (on, at) = GbPalette::step(on, at, true);
            seen.push((on, at.index()));
        }
        assert_eq!(seen[0], (true, 0));
        assert_eq!(seen[GbPalette::COUNT - 1], (true, GbPalette::COUNT - 1));
        assert!(!seen[GbPalette::COUNT].0, "the ring closes on Off");
        let (on, at) = GbPalette::step(false, at, false);
        assert_eq!((on, at.index()), (true, GbPalette::COUNT - 1), "left of Off is the last");
    }
}
