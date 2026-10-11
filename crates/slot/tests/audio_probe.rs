//! Host probe: drives the 160p suite into its sound test and reports the audio level.
//! Run: PROBE_ROM=rom.gba PROBE_CORE=/path/core.so [PROBE_DOWNS=8] cargo test -p slot --test audio_probe -- --ignored --nocapture
//! Finding (2026-10-10): with the 160p test ROM, mGBA peaks ~9200 while A is held on a tone, gpSP stays 0 (host x86 too).
use slot_retro::{ButtonMask, LibretroCore, RetroCore};
use std::path::PathBuf;

fn peak(core: &mut LibretroCore, frames: u32, mask: u16) -> u32 {
    let mut p = 0u32;
    for _ in 0..frames {
        core.run_frame(ButtonMask(mask));
        let a = core.take_audio();
        p = p.max(a.iter().map(|s| s.unsigned_abs() as u32).max().unwrap_or(0));
    }
    p
}

#[test]
#[ignore]
fn sound_test_level() {
    let lib = PathBuf::from(std::env::var("PROBE_CORE").expect("PROBE_CORE"));
    let rom = PathBuf::from(std::env::var("PROBE_ROM").expect("PROBE_ROM"));
    let dir = std::env::temp_dir();
    let mut core = LibretroCore::open_with(&lib, &dir, &dir).expect("open core");
    core.load(&rom).expect("load rom");
    println!("rate {:?}", core.av_info());
    println!("boot peak     {}", peak(&mut core, 300, 0));
    let tap = |core: &mut LibretroCore, m: u16| {
        peak(core, 5, m);
        peak(core, 25, 0)
    };
    tap(&mut core, ButtonMask::B);
    println!("menu peak     {}", peak(&mut core, 60, 0));
    tap(&mut core, ButtonMask::RIGHT);
    let downs: u32 = std::env::var("PROBE_DOWNS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    for _ in 0..downs {
        tap(&mut core, ButtonMask::DOWN);
    }
    println!("cursor peak   {}", peak(&mut core, 30, 0));
    tap(&mut core, ButtonMask::A);
    println!("submenu peak  {}", peak(&mut core, 120, 0));
    println!("held A peak   {}", peak(&mut core, 120, ButtonMask::A));
}
