# Changelog

## [slot for Android 1.0.0](https://github.com/fragilesilver/slot-for-Android/releases/tag/android-v1.0.0)

First release of the Android port, built on slot 1.6.0.

**The app**
- Runs slot on an Android phone or foldable: the shelf, cartridges, slot animation, cores (mGBA and gpSP), saves, states, rewind, fast-forward and the quick menu.
- On-screen controls laid out for the screen (side bars, below the picture, or floating over it when unfolded), plus gamepad and keyboard. The shoulder row is L, L2, R2, R: hold R2 to fast forward (double-tap to lock it on), hold L2 to rewind. Opacity and size are adjustable; touch haptics can be switched off.
- First launch after an install: the menus no longer come up blank once All files access has been granted.
- Fullscreen, with the Menu button at the top centre. Leaving the app and coming back keeps everything as it was.
- Games and everything else live in `/sdcard/slot/` once All files access is granted.
- Pause menu in game: save / load state, browse states, change core, controls, screen, cheats, achievements, back to shelf.

**Added from the forks**
- Frame blending, colour grades (seven, in linear light) and shader strength.
- GB Palettes row steps through Off and all 48 named palettes (Grayscale, DMG Green, GB Pocket, GB Light, GBC Brown, GBC Red, SGB ...) with Left / Right, and applies at once in a running game.
- Cheat codes per game (mGBA core). Fixed: codes never reached the core while RetroAchievements tracking wrapped it, and a code switched off was still applied. Switching codes on and off no longer freezes the game (mGBA left its master-code hook behind when the list was cleared; patched in `cores/mgba`). The Cheats screen now says whether the master code is running or the game never reached it (a code for another version of the game).
- RetroAchievements with offline cache and sync, and an Achievements menu: status, enable, username, password (on-screen keyboard), log out.
- Achievements: a banner names the game when it starts ("51 achievements, 0 earned", or that this ROM has none); unlock banners carry the description; Achievements > This game lists every achievement with its points and whether you have earned it.
- Automatic cartridge labels from LaunchBox, fetched in the background.
- Favourites (star a cart, show only favourites) and a letter index you can tap.

**Added for Android**
- Edit Cart: shell colour (presets, hue, vividness, shade), finish and outline, with a live preview. Saved to `Labels/cart_shell.txt`, the same file Cart Studio writes.
- Edit Cart, Picture: use a picture from your gallery (or the newest PNG/JPEG in Downloads; Left/Right switches) as the cart's label, of any size or shape. A on the row opens the gallery. It is scaled to fill the label and centre-cropped. The label it replaces is kept as `<name>.png.bak`, and Back to default puts it back.
- Edit Cart, Text: type your own words for the label on the on-screen keyboard.
- Vibration (Controls page): how strong the motor runs, 30 to 100%, for touch ticks and game rumble.
- About sticker now reads "Forked in Brunei Darussalam, by a cat with pink headphone".
- Fixed: touches on the on-screen keyboard and menus landed up to a key off toward the screen edges (the picture is drawn at whole-number scale and touches were mapped with a fractional one).
- Library: back up and restore saves, states, cheat toggles and cart colours as a zip in `Backups/` (a restore keeps a before-restore zip), and import games from the Download folder (loose files and zips).
- X and Y buttons (turbo, A or B), headphone detection, battery and clock from the phone, app icon and About screen with the device model.

**Not in this release**
- Link cable and wireless adapter play.
- Custom label art from the gallery (drop a PNG in `Labels/<platform>/` instead).
- Free-form layout editing for the on-screen controls.
