# Site media

Every clip and still the site shows is named here. Until a file exists, the site shows a gray
placeholder card with the file name and what it should show, so files can be dropped in one at a time
without touching `index.html`.

## Formats

- **Clips** are `.mp4` (H.264, no audio track needed, they play muted and loop) plus a `.webp`
  poster with the same name, which is the first frame. The poster also shows when the visitor has
  reduced motion on.
- **Phone clips** fill the landscape phone frame's screen, **1020 x 450** (about 20.4:9). A
  straight screen recording of a phone in landscape fits, since the frame crops to fill.
- **Fold clips** fill the unfolded foldable's screen, **940 x 840** (about 1.12:1).
- Keep each clip short (5 to 15 s) and under about 2 MB.

## Clips

| File | Frame | Where | What to show |
|---|---|---|---|
| `main.mp4` | phone | Hero | Shelf, insert a cart, a few seconds of play |
| `carousel.mp4` | phone | Guide: Browse the Shelf | Flick along the shelf, switch platforms with L / R, tap the letter row |
| `favourites.mp4` | phone | Guide: Favourites | Star a cart with B, hold B for favourites only |
| `core-picker.mp4` | phone | Guide: Pick an Emulator | START on a GBA cart, swap the chip, A to keep |
| `insert.mp4` | phone | Guide: Insert and Play | Tap A, the cart slides in, the game resumes |
| `pause-menu.mp4` | phone | Guide: The Pause Menu | Tap MENU at the top centre, scroll the pause menu, B to close |
| `eject.mp4` | phone | Guide: Ejecting a Game | Hold MENU, the cart pops out back to the shelf |
| `states.mp4` | phone | Guide: Save States | SELECT + R saves, SELECT + L loads, then Browse States |
| `rewind.mp4` | phone | Guide: Fast-Forward and Rewind | Hold R2, double tap R2 to lock, hold L2 |
| `controls.mp4` | phone | Guide: On-Screen Controls | Pause menu > Controls: opacity and size changing live |
| `fold.mp4` | fold | Guide: Fold and Unfold | Unfold mid-game, the layout reflows and the controls float over the picture |
| `gbc-stretch.mp4` | phone | Guide: Aspect Ratio | A GBC game: L stretches, R puts it back |
| `shaders.mp4` | phone | Guide: Screen | SELECT + A cycling shaders, then the Screen page |
| `palettes.mp4` | phone | Guide: Game Boy Palettes | A GB game: SELECT + R2 stepping through palettes |
| `edit-cart.mp4` | phone | Guide: Edit Cart | Change the shell colour, pick a gallery photo, type text |
| `cheats.mp4` | phone | Guide: Cheats | Pause menu > Cheats, toggling a code |
| `achievements.mp4` | phone | Guide: RetroAchievements | Shelf menu > Achievements sign-in, then an unlock in game |
| `library.mp4` | phone | Guide: Library | Back up, restore, import from Downloads |
| `settings.mp4` | phone | Guide: Settings | MENU on the shelf, scroll the shelf menu |

Each needs its `.webp` poster too (`main.webp`, `carousel.webp`, and so on).

## Stills

| File | Aspect | Where | What to show |
|---|---|---|---|
| `all-files-access.webp` | portrait phone, 9:20 | Installing, step 3 | Android's All files access screen with slot switched on |
| `layout-phone.webp` | 1020 x 450 | Buttons > On Screen | The on-screen controls on a phone in landscape |
| `layout-fold.webp` | 940 x 840 | Buttons > On Screen | The on-screen controls on an unfolded foldable |

## Also here

- `slot.mp3`: the pronunciation, played by the speaker button next to the wordmark.
- `record.sh` and `clips/*.txt`: upstream's recorder. It renders 720 x 480 game-screen clips off-device
  with `cargo run --example record -p slot`. That's handy for screen content, but it doesn't draw the
  Android touch overlay, so on-device screen recordings are the better source for this site.
