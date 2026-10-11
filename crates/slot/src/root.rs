use std::path::{Path, PathBuf};

pub const DIRS: [&str; 21] = [
    "BIOS",
    "Cheats",
    "Config",
    "Games",
    "Games/GBA",
    "Games/GB",
    "Games/GBC",
    "Labels",
    "Labels/GBA",
    "Labels/GB",
    "Labels/GBC",
    "Saves",
    "Saves/GBA",
    "Saves/GB",
    "Saves/GBC",
    "States",
    "States/GBA",
    "States/GB",
    "States/GBC",
    "System",
    "Wallpapers",
];

pub fn ensure(root: &Path) {
    for sub in DIRS {
        let _ = std::fs::create_dir_all(root.join(sub));
    }
    if let Err(e) = slot_store::move_config(root) {
        eprintln!("slot: config: {e}");
    }
}

pub fn bios_dir(root: &Path) -> PathBuf {
    root.join("BIOS")
}

const BIOS_FILE: &str = "gba_bios.bin";

const BIOS_BYTES: u64 = 16 * 1024;
const BIOS_FIRST_BYTE: u8 = 0x18;

pub fn has_real_bios(root: &Path) -> bool {
    let Ok(mut f) = std::fs::File::open(bios_dir(root).join(BIOS_FILE)) else {
        return false;
    };
    if !f.metadata().is_ok_and(|m| m.len() == BIOS_BYTES) {
        return false;
    }
    let mut first = [0u8; 1];
    std::io::Read::read_exact(&mut f, &mut first).is_ok() && first[0] == BIOS_FIRST_BYTE
}

pub fn saves_dir(root: &Path) -> PathBuf {
    root.join("Saves")
}

/// One cheat code for a cart: `code` goes to the core verbatim, `desc` is what the menu shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheatEntry {
    pub desc: String,
    pub code: String,
}

/// `Cheats/<stem>.txt`: one code per line, `#` starts a description (`CODE # Infinite lives`),
/// blank and `#`-only lines are ignored. A multi-line GameShark / Action Replay code is joined
/// with `+` on one line. Missing or unreadable means no cheats.
pub fn cheats(root: &Path, stem: &str) -> Vec<CheatEntry> {
    let Ok(text) = std::fs::read_to_string(root.join("Cheats").join(format!("{stem}.txt"))) else {
        return Vec::new();
    };
    parse_cheats(&text)
}

pub fn parse_cheats(text: &str) -> Vec<CheatEntry> {
    text.lines()
        .filter_map(|line| {
            let (code, desc) = match line.split_once('#') {
                Some((code, desc)) => (code.trim(), desc.trim()),
                None => (line.trim(), ""),
            };
            if code.is_empty() {
                return None;
            }
            Some(CheatEntry {
                desc: if desc.is_empty() {
                    code.to_owned()
                } else {
                    desc.to_owned()
                },
                code: code.to_owned(),
            })
        })
        .collect()
}

/// Which of `list` were left on last time (`Cheats/<stem>.on`, one enabled code per line).
pub fn cheats_on(root: &Path, stem: &str, list: &[CheatEntry]) -> Vec<bool> {
    let text = std::fs::read_to_string(root.join("Cheats").join(format!("{stem}.on")))
        .unwrap_or_default();
    list.iter()
        .map(|c| text.lines().any(|l| l.trim() == c.code))
        .collect()
}

pub fn save_cheats_on(root: &Path, stem: &str, flags: &[(bool, &str)]) {
    let text: String = flags
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, code)| format!("{code}\n"))
        .collect();
    let _ = std::fs::write(root.join("Cheats").join(format!("{stem}.on")), text);
}

#[cfg(test)]
mod cheat_tests {
    use super::*;

    #[test]
    fn parses_code_description_and_skips_noise() {
        let list = parse_cheats("# header\n\nDEADBEEF 0001 # Infinite lives\n  ABCD1234  \n");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].code, "DEADBEEF 0001");
        assert_eq!(list[0].desc, "Infinite lives");
        assert_eq!(list[1].desc, "ABCD1234");
    }
}
