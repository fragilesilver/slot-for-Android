//! Pulling games in from the phone's Downloads folder: loose `.gba` / `.gb` / `.gbc` files and
//! `.zip` archives holding them are copied into `Games/<platform>/`. Nothing is moved or
//! deleted, and a game that is already there is left alone.
use std::io;
use std::path::{Path, PathBuf};

use slot_store::Platform;

use crate::backup::{zip_entries, ZipEntry};

/// The biggest game worth copying (a GBA cart is at most 32 MB).
const MAX_GAME: u64 = 64 * 1024 * 1024;
/// The biggest archive that is opened; zips are read whole, and a console-set bundle of
/// hundreds of megabytes is not a game.
const MAX_ZIP: u64 = 128 * 1024 * 1024;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Imported {
    pub added: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// Where downloads land: `Download/` beside the library's own folder on shared storage.
pub fn downloads_dir(root: &Path) -> Option<PathBuf> {
    let d = root.parent()?.join("Download");
    d.is_dir().then_some(d)
}

fn platform_of(name: &str) -> Option<Platform> {
    // `.gb` and `.gbc` are both accepted by either Game Boy shelf; the extension picks the one
    // the game belongs on.
    let ext = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "gba" => Some(Platform::Gba),
        "gb" => Some(Platform::Gb),
        "gbc" => Some(Platform::Gbc),
        _ => None,
    }
}

fn plain_name(name: &str) -> Option<&str> {
    let leaf = name.rsplit('/').next()?;
    (!leaf.is_empty() && !leaf.starts_with('.')).then_some(leaf)
}

fn place(root: &Path, leaf: &str, platform: Platform, bytes: impl FnOnce() -> io::Result<Vec<u8>>, out: &mut Imported) {
    let dest = root.join("Games").join(platform.dir_name()).join(leaf);
    if dest.exists() {
        out.skipped += 1;
        return;
    }
    let written = bytes().and_then(|b| {
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = dest.with_extension("part");
        std::fs::write(&tmp, b)?;
        std::fs::rename(&tmp, &dest)
    });
    match written {
        Ok(()) => out.added += 1,
        Err(e) => {
            eprintln!("slot: import {leaf}: {e}");
            out.failed += 1;
        }
    }
}

/// Import everything it can from `from` into the library at `root`.
pub fn from_folder(root: &Path, from: &Path) -> Imported {
    let mut out = Imported::default();
    let Ok(entries) = std::fs::read_dir(from) else {
        return out;
    };
    let mut files: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
    files.sort();
    for path in files {
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
            continue;
        };
        if name.starts_with('.') || !path.is_file() {
            continue;
        }
        let len = path.metadata().map_or(0, |m| m.len());
        if let Some(platform) = platform_of(&name) {
            if len > MAX_GAME {
                continue;
            }
            place(root, &name, platform, || std::fs::read(&path), &mut out);
        } else if name.to_lowercase().ends_with(".zip") {
            if len > MAX_ZIP {
                continue;
            }
            let Ok(data) = std::fs::read(&path) else {
                out.failed += 1;
                continue;
            };
            let Ok(entries) = zip_entries(&data) else {
                out.failed += 1;
                continue;
            };
            let games: Vec<(&ZipEntry, &str, Platform)> = entries
                .iter()
                .filter_map(|e| {
                    let leaf = plain_name(&e.name)?;
                    Some((e, leaf, platform_of(leaf)?))
                })
                .collect();
            for (entry, leaf, platform) in games {
                place(root, leaf, platform, || entry.unpack(&data), &mut out);
            }
        }
    }
    out
}

/// The most recently saved picture (PNG or JPEG) in `dir`.
pub fn newest_picture(dir: &Path) -> Option<PathBuf> {
    const MAX_PICTURE: u64 = 24 * 1024 * 1024;
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let lower = name.to_lowercase();
            if name.starts_with('.')
                || !(lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg"))
            {
                return None;
            }
            let meta = e.metadata().ok()?;
            (meta.is_file() && meta.len() <= MAX_PICTURE).then(|| (meta.modified().ok(), e.path()))
        })
        .max_by_key(|(when, _)| *when)
        .map(|(_, path)| path)
}

/// Make `picture` the label of the cart `stem` on `platform`. A label that was there is kept
/// beside it as `<stem>.png.bak` (the first time only), so the scraped art can be put back.
pub fn set_label(root: &Path, platform: Platform, stem: &str, picture: &Path) -> Result<PathBuf, String> {
    let bytes = std::fs::read(picture).map_err(|e| e.to_string())?;
    let png = slot_labels::label_from_picture(&bytes, platform).map_err(|e| e.to_string())?;
    store_label(root, platform, stem, &png)
}

/// Write `png` as the label of `stem`, remembering what was there (see `set_label`).
pub fn store_label(root: &Path, platform: Platform, stem: &str, png: &[u8]) -> Result<PathBuf, String> {
    let dir = root.join("Labels").join(platform.dir_name());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(format!("{stem}.png"));
    // Remember what was here before the first custom picture, so "Back to default" can put it
    // back: the old label as `.png.bak`, or an empty `.png.none` when there was no label.
    let bak = dir.join(format!("{stem}.png.bak"));
    let none = dir.join(format!("{stem}.png.none"));
    if !bak.exists() && !none.exists() {
        if dest.exists() {
            let _ = std::fs::copy(&dest, &bak);
        } else {
            let _ = std::fs::write(&none, b"");
        }
    }
    let tmp = dir.join(format!("{stem}.png.part"));
    std::fs::write(&tmp, png).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &dest).map_err(|e| e.to_string())?;
    Ok(dest)
}

/// Undo `set_label`: the label the cart had before its first custom picture comes back (or goes,
/// if it had none). Returns the label file now in place, `None` when the cart has no label.
/// A cart that never had a picture set is left alone.
pub fn restore_label(root: &Path, platform: Platform, stem: &str) -> Option<PathBuf> {
    let dir = root.join("Labels").join(platform.dir_name());
    let dest = dir.join(format!("{stem}.png"));
    let bak = dir.join(format!("{stem}.png.bak"));
    let none = dir.join(format!("{stem}.png.none"));
    if bak.exists() {
        if std::fs::copy(&bak, &dest).is_ok() {
            let _ = std::fs::remove_file(&bak);
        }
    } else if none.exists() {
        let _ = std::fs::remove_file(&dest);
        let _ = std::fs::remove_file(&none);
    }
    dest.exists().then_some(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_photo_of_any_size_becomes_a_label_and_the_old_label_is_kept() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().join("slot");
        let pic = d.path().join("photo.jpg");
        let img = image::RgbImage::from_fn(900, 1200, |x, y| image::Rgb([(x / 4) as u8, (y / 5) as u8, 90]));
        img.save(&pic).unwrap();
        std::fs::create_dir_all(root.join("Labels/GBA")).unwrap();
        std::fs::write(root.join("Labels/GBA/Game.png"), b"old").unwrap();
        let dest = set_label(&root, Platform::Gba, "Game", &pic).unwrap();
        let out = image::open(&dest).unwrap();
        assert_eq!((out.width(), out.height()), slot_labels::label_size(Platform::Gba));
        assert_eq!(std::fs::read(root.join("Labels/GBA/Game.png.bak")).unwrap(), b"old");
        assert_eq!(newest_picture(d.path()).as_deref(), Some(pic.as_path()));
        // A second picture keeps the first original; restoring brings it back.
        set_label(&root, Platform::Gba, "Game", &pic).unwrap();
        assert_eq!(restore_label(&root, Platform::Gba, "Game"), Some(dest.clone()));
        assert_eq!(std::fs::read(&dest).unwrap(), b"old");
        assert!(!root.join("Labels/GBA/Game.png.bak").exists());
        // With no label to begin with, restoring removes the picture.
        set_label(&root, Platform::Gba, "Bare", &pic).unwrap();
        assert!(root.join("Labels/GBA/Bare.png").exists());
        assert_eq!(restore_label(&root, Platform::Gba, "Bare"), None);
        assert!(!root.join("Labels/GBA/Bare.png").exists());
    }

    #[test]
    fn games_come_in_from_loose_files_and_zips_and_are_never_overwritten() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().join("slot");
        let from = d.path().join("Download");
        std::fs::create_dir_all(&from).unwrap();
        std::fs::write(from.join("Loose.gba"), b"GBA").unwrap();
        std::fs::write(from.join("Mono.GB"), b"GB").unwrap();
        std::fs::write(from.join("notes.txt"), b"x").unwrap();
        let pack = d.path().join("pack");
        std::fs::create_dir_all(&pack).unwrap();
        std::fs::write(pack.join("Zipped.gbc"), vec![5u8; 5000]).unwrap();
        std::fs::write(pack.join("readme.txt"), b"hi").unwrap();
        crate::backup::write_zip(
            &pack,
            &["Zipped.gbc".to_string(), "readme.txt".to_string()],
            &from.join("bundle.zip"),
            0,
        )
        .unwrap();
        std::fs::write(from.join("Already.gba"), b"new").unwrap();
        std::fs::create_dir_all(root.join("Games/GBA")).unwrap();
        std::fs::write(root.join("Games/GBA/Already.gba"), b"old").unwrap();
        let got = from_folder(&root, &from);
        assert_eq!(got, Imported { added: 3, skipped: 1, failed: 0 });
        assert_eq!(std::fs::read(root.join("Games/GBC/Zipped.gbc")).unwrap(), vec![5u8; 5000]);
        assert_eq!(std::fs::read(root.join("Games/GBA/Loose.gba")).unwrap(), b"GBA");
        assert_eq!(std::fs::read(root.join("Games/GB/Mono.GB")).unwrap(), b"GB");
        assert_eq!(std::fs::read(root.join("Games/GBA/Already.gba")).unwrap(), b"old");
        assert_eq!(from_folder(&root, &from).added, 0);
    }
}
