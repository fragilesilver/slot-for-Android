# slot for Android

An Android port of **slot**, the Game Boy / Game Boy Color / Game Boy Advance frontend.
Package id `com.fragilesilver.slot`. Forked in Brunei Darussalam, by a cat with pink headphone.

A fork of [BrandonKowalski/slot](https://github.com/BrandonKowalski/slot). Claude (Anthropic), through Claude Code,
helped with forking it to Android: the shell, touch controls, Edit Cart, library, packaging and these docs. I directed
it, tested it on my own Samsung Z Fold 7 and decided what shipped.

## Credits

- **slot** is by **Brandon T. Kowalski** (<https://github.com/BrandonKowalski/slot>), GPL-3.0-or-later.
  Everything here is built on that code; the Android shell, touch controls, pause menu,
  storage and power glue are the port.
- **pvaibhav's fork** (<https://github.com/pvaibhav/slot>): the RetroAchievements integration,
  the LaunchBox cartridge-art fetcher and the frame-blending idea were taken from it.
- **RetroGame66's fork** (<https://github.com/RetroGame66/slot>): the designs for cheat codes,
  colour-correction grades and panel-mask presets, plus the favourites / letter-index ideas,
  come from it. The Android code for these is written fresh against slot's own structure.
- **mGBA** (MPL-2.0) and **gpSP** (GPL-2.0) do the emulation, with the patches listed under `cores/`.
- **rcheevos** (MIT) is vendored under `crates/slot-achievements`.
- Type is Open Sans and Nerd Font symbols; the panel mask is derived from Gigaherz's LCD3x.

## Setting up

1. Install the APK and open it. When asked, allow **All files access**; slot then keeps everything
   in `/sdcard/slot/` where a file manager or a cable can reach it.
2. Put your games in `Games/GBA`, `Games/GB` and `Games/GBC` (unzipped `.gba`, `.gb`, `.gbc`),
   or use **Library > Import games from Downloads**.
3. Open slot again if you added files by hand; the shelf is built when the app starts.

## Using it

- **Shelf:** left / right to browse, A to insert a cart, B to star it (hold B for favourites only),
  tap the letter row to jump. Menu opens the shelf menu.
- **Shelf menu:** Favourite, Favourites Only, Edit Cart, Screen, Gameplay, Achievements, Library,
  Date & Time, About.
- **In game:** Menu opens the pause menu (save / load state, states, core, controls, screen,
  cheats, achievements, back to shelf).
- **Controls page:** turbo, what X and Y do, controls opacity and size, touch haptics.
- **Edit Cart:** change the shell colour, finish and outline of the cart under the caret; the
  choice is written to `Labels/cart_shell.txt` (the file Cart Studio produces, which wins over
  `Config/cart_shell.txt`). **Picture** opens your gallery (Left/Right switches it to the newest PNG/JPEG in
  `Download`) and fits the photo to the label (any size; centre-cropped). **Text** opens the keyboard and puts your words on the label. Back to default puts
  the original label back.
- **Library:** back up now, restore the latest backup, import games from Download.
- **Cheats:** `Cheats/<rom name>.txt`, one `CODE # name` per line (mGBA core only; gpSP has none).
- **RetroAchievements:** shelf menu > Achievements. Changes apply to the next game you start.
- **Cartridge labels:** fetched automatically when online; drop `Labels/<platform>/<rom name>.png`
  to use your own.

## Folders

`Games` `Saves` `States` `Labels` `Cheats` `Config` `Backups` `BIOS` `System` `Wallpapers`.
Backups hold `Saves`, `States`, `Cheats` and `Labels/cart_shell.txt`; they leave out `Config/`
(your RetroAchievements login) on purpose.

## Not in this release

Link play, custom label art from the gallery, a free-form layout editor for the controls.

## Building

```
eval "$(mise env)"
SLOT_RELEASE=1 android/build.sh run      # signed release, installs on the connected device
android/build.sh run                     # debug
```

Release signing uses `~/.android/slot-release.jks` and `~/.android/slot-release.password`.
Back both up: without them an update has to be installed over an uninstall.
Needs Rust (see `rust-toolchain.toml`), cargo-ndk, cargo-apk, NDK r27 and a JDK (javac, for the small
`android/java` helper that opens the gallery; `android/build-dex.sh` packs it into the APK).
