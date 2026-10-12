//! Backing up and restoring what is only on the phone: saves, save states, cheat toggles, the
//! cart shells and the unsynced achievement unlocks. A backup is an ordinary `.zip` in
//! `Backups/`, so the Files app (or a cable) can carry it off the device.
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;

pub const DIR: &str = "Backups";
const PREFIX: &str = "slot-backup-";
const BEFORE: &str = "slot-before-restore-";
const MAX_FILE: u64 = 256 * 1024 * 1024;

/// Folders that go into a backup. Achievement caches and login tokens are left out; the unlock
/// ledger is the one thing in there that cannot be fetched again.
const TREES: [&str; 4] = ["Saves", "States", "Cheats", "Labels/cart_shell.txt"];

fn wanted(rel: &str) -> bool {
    if rel.starts_with("Saves/RetroAchievements/") {
        return rel.ends_with("/unlocks.json");
    }
    true
}

fn walk(root: &Path, rel: &str, out: &mut Vec<String>) {
    let path = root.join(rel);
    let Ok(meta) = std::fs::symlink_metadata(&path) else {
        return;
    };
    if meta.is_file() {
        if wanted(rel) {
            out.push(rel.to_string());
        }
    } else if meta.is_dir() {
        let Ok(entries) = std::fs::read_dir(&path) else {
            return;
        };
        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter(|n| !n.starts_with('.'))
            .collect();
        names.sort();
        for name in names {
            walk(root, &format!("{rel}/{name}"), out);
        }
    }
}

pub fn files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for tree in TREES {
        walk(root, tree, &mut out);
    }
    out
}

fn civil(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400) as u32;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d, rem / 3600, rem % 3600 / 60, rem % 60)
}

fn stamp(secs: i64) -> String {
    let (y, mo, d, h, mi, s) = civil(secs);
    format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}")
}

fn dos_time(secs: i64) -> (u16, u16) {
    let (y, mo, d, h, mi, s) = civil(secs);
    let y = (y.clamp(1980, 2107) - 1980) as u16;
    (
        ((h as u16) << 11) | ((mi as u16) << 5) | (s as u16 / 2),
        (y << 9) | ((mo as u16) << 5) | d as u16,
    )
}

struct Entry {
    name: String,
    crc: u32,
    packed: u32,
    size: u32,
    offset: u32,
}

fn put16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// Zip up `files` (paths relative to `root`) into `dest`. Returns how many went in.
pub(crate) fn write_zip(root: &Path, files: &[String], dest: &Path, secs: i64) -> io::Result<usize> {
    let (time, date) = dos_time(secs);
    let mut out: Vec<u8> = Vec::new();
    let mut entries = Vec::new();
    for name in files {
        let data = match std::fs::read(root.join(name)) {
            Ok(d) => d,
            Err(_) => continue,
        };
        if data.len() as u64 > u32::MAX as u64 / 2 {
            continue;
        }
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::fast());
        enc.write_all(&data)?;
        let packed = enc.finish()?;
        let crc = crc32fast::hash(&data);
        let offset = out.len() as u32;
        put32(&mut out, 0x0403_4b50);
        put16(&mut out, 20);
        put16(&mut out, 0x0800);
        put16(&mut out, 8);
        put16(&mut out, time);
        put16(&mut out, date);
        put32(&mut out, crc);
        put32(&mut out, packed.len() as u32);
        put32(&mut out, data.len() as u32);
        put16(&mut out, name.len() as u16);
        put16(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&packed);
        entries.push(Entry {
            name: name.clone(),
            crc,
            packed: packed.len() as u32,
            size: data.len() as u32,
            offset,
        });
    }
    let dir_at = out.len() as u32;
    for e in &entries {
        put32(&mut out, 0x0201_4b50);
        put16(&mut out, 20);
        put16(&mut out, 20);
        put16(&mut out, 0x0800);
        put16(&mut out, 8);
        put16(&mut out, time);
        put16(&mut out, date);
        put32(&mut out, e.crc);
        put32(&mut out, e.packed);
        put32(&mut out, e.size);
        put16(&mut out, e.name.len() as u16);
        put16(&mut out, 0);
        put16(&mut out, 0);
        put16(&mut out, 0);
        put16(&mut out, 0);
        put32(&mut out, 0);
        put32(&mut out, e.offset);
        out.extend_from_slice(e.name.as_bytes());
    }
    let dir_len = out.len() as u32 - dir_at;
    put32(&mut out, 0x0605_4b50);
    put16(&mut out, 0);
    put16(&mut out, 0);
    put16(&mut out, entries.len() as u16);
    put16(&mut out, entries.len() as u16);
    put32(&mut out, dir_len);
    put32(&mut out, dir_at);
    put16(&mut out, 0);
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = dest.with_extension("zip.tmp");
    std::fs::write(&tmp, &out)?;
    std::fs::rename(&tmp, dest)?;
    Ok(entries.len())
}

/// Write a backup named for the time `secs`; returns its path and the number of files in it.
pub fn create(root: &Path, secs: i64) -> io::Result<(PathBuf, usize)> {
    named(root, PREFIX, secs)
}

fn named(root: &Path, prefix: &str, secs: i64) -> io::Result<(PathBuf, usize)> {
    let dest = root.join(DIR).join(format!("{prefix}{}.zip", stamp(secs)));
    let n = write_zip(root, &files(root), &dest, secs)?;
    Ok((dest, n))
}

/// The newest backup, by the time in its name.
pub fn latest(root: &Path) -> Option<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(root.join(DIR))
        .ok()?
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| {
            p.extension().is_some_and(|x| x == "zip")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(PREFIX))
        })
        .collect();
    found.sort();
    found.pop()
}

fn le16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn le32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn bad(why: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, why.to_string())
}

/// A name from inside an archive that is safe to join onto the library root.
fn safe(name: &str) -> bool {
    let top = name.split('/').next().unwrap_or("");
    if !TREES.iter().any(|t| t.split('/').next() == Some(top)) {
        return false;
    }
    Path::new(name)
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
}

/// One file inside a zip archive.
pub(crate) struct ZipEntry {
    pub name: String,
    method: u16,
    crc: u32,
    start: usize,
    packed: usize,
    size: u64,
}

impl ZipEntry {
    pub fn size(&self) -> u64 {
        self.size
    }

    /// The file's bytes, inflated and checked against the archive's checksum.
    pub fn unpack(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        let body = data
            .get(self.start..self.start + self.packed)
            .ok_or_else(|| bad("short zip"))?;
        let bytes = match self.method {
            0 => body.to_vec(),
            8 => {
                let mut v = Vec::with_capacity(self.size as usize);
                DeflateDecoder::new(body).take(MAX_FILE).read_to_end(&mut v)?;
                v
            }
            m => return Err(bad(&format!("compression method {m} is not supported"))),
        };
        if crc32fast::hash(&bytes) != self.crc {
            return Err(bad(&format!("{} failed its checksum", self.name)));
        }
        Ok(bytes)
    }
}

/// The central directory of the zip held in `data`.
pub(crate) fn zip_entries(data: &[u8]) -> io::Result<Vec<ZipEntry>> {
    let eocd = data
        .windows(4)
        .rposition(|w| w == [0x50, 0x4b, 0x05, 0x06])
        .ok_or_else(|| bad("not a zip file"))?;
    let short = || bad("short zip");
    let count = le16(data, eocd + 10).ok_or_else(short)? as usize;
    let mut at = le32(data, eocd + 16).ok_or_else(short)? as usize;
    let mut out = Vec::new();
    for _ in 0..count {
        if le32(data, at) != Some(0x0201_4b50) {
            return Err(bad("damaged zip directory"));
        }
        let method = le16(data, at + 10).ok_or_else(short)?;
        let crc = le32(data, at + 16).ok_or_else(short)?;
        let packed = le32(data, at + 20).ok_or_else(short)? as usize;
        let size = le32(data, at + 24).ok_or_else(short)? as u64;
        let n = le16(data, at + 28).ok_or_else(short)? as usize;
        let m = le16(data, at + 30).ok_or_else(short)? as usize;
        let k = le16(data, at + 32).ok_or_else(short)? as usize;
        let local = le32(data, at + 42).ok_or_else(short)? as usize;
        let name = String::from_utf8_lossy(data.get(at + 46..at + 46 + n).ok_or_else(short)?)
            .into_owned();
        at += 46 + n + m + k;
        let ln = le16(data, local + 26).ok_or_else(short)? as usize;
        let lm = le16(data, local + 28).ok_or_else(short)? as usize;
        out.push(ZipEntry {
            name,
            method,
            crc,
            start: local + 30 + ln + lm,
            packed,
            size,
        });
    }
    Ok(out)
}

/// Unpack `zip` over the library. Returns the number of files written.
pub fn restore_from(root: &Path, zip: &Path) -> io::Result<usize> {
    let data = std::fs::read(zip)?;
    let mut written = 0;
    for entry in zip_entries(&data)? {
        if entry.name.ends_with('/') || !safe(&entry.name) || entry.size() > MAX_FILE {
            continue;
        }
        if !matches!(entry.method, 0 | 8) {
            continue;
        }
        let bytes = entry.unpack(&data)?;
        let dest = root.join(&entry.name);
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&dest, bytes)?;
        written += 1;
    }
    Ok(written)
}

/// Restore the newest backup, first setting today's files aside in `slot-before-restore-*.zip`
/// so a wrong restore can be undone. Returns the number of files restored.
pub fn restore_latest(root: &Path, secs: i64) -> io::Result<usize> {
    let zip = latest(root).ok_or_else(|| bad("no backup yet"))?;
    let _ = named(root, BEFORE, secs)?;
    restore_from(root, &zip)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(root: &Path, rel: &str, bytes: &[u8]) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }

    #[test]
    fn a_backup_restores_what_was_lost_and_leaves_secrets_out() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        put(r, "Saves/GBA/Game.sav", &[7u8; 40_000]);
        put(r, "States/GBA/Game.state1", &[9u8; 100]);
        put(r, "Saves/RetroAchievements/ab/unlocks.json", b"[1]");
        put(r, "Saves/RetroAchievements/ab/auth.json", b"secret");
        put(r, "Config/retroachievements.toml", b"password");
        let (zip, n) = create(r, 1_760_000_000).unwrap();
        assert_eq!(n, 3);
        assert!(zip.file_name().unwrap().to_str().unwrap().starts_with("slot-backup-2025"));
        std::fs::remove_file(r.join("Saves/GBA/Game.sav")).unwrap();
        put(r, "States/GBA/Game.state1", b"changed");
        assert_eq!(restore_latest(r, 1_760_000_100).unwrap(), 3);
        assert_eq!(std::fs::read(r.join("Saves/GBA/Game.sav")).unwrap(), vec![7u8; 40_000]);
        assert_eq!(std::fs::read(r.join("States/GBA/Game.state1")).unwrap(), vec![9u8; 100]);
        let before = std::fs::read_dir(r.join(DIR)).unwrap().count();
        assert_eq!(before, 2, "the files that were replaced are kept as a backup too");
    }

    #[test]
    fn names_that_climb_out_of_the_library_are_refused() {
        assert!(safe("Saves/GBA/a.sav"));
        assert!(safe("Labels/cart_shell.txt"));
        assert!(!safe("../x"));
        assert!(!safe("Saves/../../x"));
        assert!(!safe("/etc/passwd"));
        assert!(!safe("Config/slot.state"));
    }

    #[test]
    fn dates_come_out_right() {
        assert_eq!(stamp(0), "19700101-000000");
        assert_eq!(stamp(1_760_000_000), "20251009-085320");
    }
}
