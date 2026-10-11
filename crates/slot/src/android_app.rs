use std::path::PathBuf;
use std::time::{Duration, Instant};

use android_activity::{AndroidApp, MainEvent, PollEvent};
use slot_gfx::{Compositor, FbdevSurface, Overlay, Surface};
use slot_power::{Platform, SimPlatform};
use slot_store::{read_slot_state, write_slot_state};

use crate::android_platform::{local_utc_offset_min, AndroidPlatform};
use crate::android_storage;

use crate::frontend::Frontend;
use crate::input::AndroidInput;

const FRAME: Duration = Duration::from_micros(16_667);

struct Gfx {
    surface: FbdevSurface,
    compositor: Compositor,
    overlay: Overlay,
}

fn data_root(app: &AndroidApp) -> PathBuf {
    app.external_data_path()
        .or_else(|| app.internal_data_path())
        .unwrap_or_else(|| PathBuf::from("/data/local/tmp/slot"))
}

fn open_gfx(app: &AndroidApp) -> Option<Gfx> {
    let win = app.native_window()?;
    let built = FbdevSurface::from_native_window(win.ptr().as_ptr().cast())
        .and_then(|s| {
            let c = Compositor::new(&s)?;
            let o = Overlay::new(&s)?;
            Ok((s, c, o))
        });
    match built {
        Ok((surface, compositor, overlay)) => {
            log::info!("egl surface up, {:?}", surface.window_size());
            Some(Gfx { surface, compositor, overlay })
        }
        Err(e) => {
            log::error!("surface: {e}");
            None
        }
    }
}

/// Prepares `root`, pins the device clock/timezone into slot's state, and boots the frontend.
fn boot_frontend(app: &AndroidApp, root: &PathBuf) -> Frontend {
    log::info!("root {}", root.display());
    crate::root::ensure(root);
    // RetroAchievements: leave an example next to where the real file goes.
    let example = root.join("Config").join("retroachievements.toml.example");
    if !example.exists() {
        let _ = std::fs::write(
            &example,
            "# Rename to retroachievements.toml, fill in your RetroAchievements account, then\n\
             # reopen slot. Achievements are softcore, cached for offline play and synced when\n\
             # the phone is online.\n\
             enabled = true\n\
             username = \"\"\n\
             password = \"\"\n\n\
             # After a successful login slot remembers a token and the password can be removed.\n\
             # token = \"\"\n",
        );
    }
    let readme = root.join("Cheats").join("README.txt");
    if !readme.exists() {
        let _ = std::fs::write(
            &readme,
            "Cheat codes (mGBA core only)\n\n\
             Make a file here named after the ROM, e.g. Games/GBA/Emerald.gba -> Emerald.txt\n\
             One code per line, then '#' and a name:\n\n\
             82003A4C 0001 # Walk through walls\n\n\
             Join a multi-line code with '+'. Open the in-game menu > Cheats to switch codes on.\n",
        );
    }
    std::env::set_var("SLOT_ROOT", root);
    if let Some(model) = unsafe { crate::android_platform::device_model(app.vm_as_ptr()) } {
        crate::device_info::set_model(&model);
    }
    if let Some(maker) = unsafe { crate::android_platform::device_maker(app.vm_as_ptr()) } {
        crate::device_info::set_maker(&maker);
    }

    // Android keeps the clock: mark it set and follow the device's timezone, so slot never
    // shows its set-the-clock screen.
    let mut state = read_slot_state(root);
    state.clock_set = true;
    state.utc_offset_min = local_utc_offset_min();
    if let Err(e) = write_slot_state(root, &state) {
        log::warn!("slot.state: {e}");
    }

    let platform: Box<dyn Platform> = match unsafe {
        AndroidPlatform::new(root.clone(), app.vm_as_ptr(), app.activity_as_ptr())
    } {
        Some(p) => Box::new(p),
        None => {
            log::warn!("no JNI; falling back to the simulated platform");
            Box::new(SimPlatform::at(root.clone()))
        }
    };
    Frontend::boot(platform)
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("slot"),
    );
    let private = data_root(&app);
    let mut root = match android_storage::all_files_granted(app.vm_as_ptr()) {
        true => {
            let shared = android_storage::shared_root();
            android_storage::migrate(&private, &shared);
            shared
        }
        false => {
            if android_storage::first_ask(&private) {
                android_storage::ask_all_files(app.vm_as_ptr(), app.activity_as_ptr());
            }
            private.clone()
        }
    };
    let mut on_shared = root != private;
    app.set_window_flags(
        android_activity::WindowManagerFlags::FULLSCREEN,
        android_activity::WindowManagerFlags::empty(),
    );
    let mut frontend = boot_frontend(&app, &root);
    let mut input = AndroidInput::new();
    let mut gfx: Option<Gfx> = None;
    // The GL context outlives the window; `shown` says a window is attached right now.
    let mut shown = false;
    let mut bezel_for = (0u32, 0u32);
    let mut quit = false;
    // Caption textures for the on-screen controls, rebuilt when the layout changes.
    let mut captions: Vec<(usize, [f32; 4], bool)> = Vec::new();
    let mut captions_for = (u32::MAX, false);

    while !quit {
        let wait = if shown { Duration::ZERO } else { Duration::from_millis(100) };
        app.poll_events(Some(wait), |event| {
            let PollEvent::Main(main) = event else { return };
            match main {
                MainEvent::InitWindow { .. } => {
                    captions.clear();
                    captions_for = (u32::MAX, false);
                    if let (Some(g), Some(win)) = (gfx.as_mut(), app.native_window()) {
                        match g.surface.attach_window(win.ptr().as_ptr().cast()) {
                            Ok(()) => {
                                shown = true;
                                let size = g.surface.window_size();
                                if size != bezel_for {
                                    frontend.upload_bezel(&mut g.compositor, size);
                                    bezel_for = size;
                                }
                            }
                            Err(e) => {
                                log::error!("reattach: {e}");
                                gfx = None;
                            }
                        }
                    }
                    if gfx.is_none() {
                        gfx = open_gfx(&app);
                        if let Some(g) = gfx.as_mut() {
                            frontend.upload_faces(&mut g.compositor);
                            let size = g.surface.window_size();
                            frontend.upload_bezel(&mut g.compositor, size);
                            bezel_for = size;
                        }
                        shown = gfx.is_some();
                    }
                }
                MainEvent::TerminateWindow { .. } => {
                    captions.clear();
                    captions_for = (u32::MAX, false);
                    if let Some(g) = gfx.as_mut() {
                        g.surface.detach_window();
                    }
                    shown = false;
                    input.release_all();
                }
                MainEvent::LostFocus => input.release_all(),
                MainEvent::Resume { .. } => {
                    // Back from the permission page: switch to the shared folder.
                    if !on_shared && android_storage::all_files_granted(app.vm_as_ptr()) {
                        let shared = android_storage::shared_root();
                        android_storage::migrate(&private, &shared);
                        root = shared;
                        on_shared = true;
                        frontend = boot_frontend(&app, &root);
                        if let Some(g) = gfx.as_mut() {
                            frontend.upload_faces(&mut g.compositor);
                            frontend.upload_bezel(&mut g.compositor, g.surface.window_size());
                        }
                    }
                }
                MainEvent::Destroy => quit = true,
                _ => {}
            }
        });
        let (true, Some(g)) = (shown, gfx.as_mut()) else {
            continue;
        };
        let began = Instant::now();
        let window = g.surface.window_size();
        input.set_window(window);
        let (opacity, size) = frontend.app().touch_style();
        input.set_style(opacity, size);
        input.set_menu_mode(frontend.app().navigating());
        input.pump(&app);
        if frontend.app_mut().take_rescan() {
            // Games were imported: boot again so the shelf is built from the new library.
            frontend = boot_frontend(&app, &root);
            frontend.upload_faces(&mut g.compositor);
            let size = g.surface.window_size();
            frontend.upload_bezel(&mut g.compositor, size);
            continue;
        }
        if let Some(dest) = frontend.app_mut().take_picker() {
            android_storage::pick_picture(app.vm_as_ptr(), app.activity_as_ptr(), &dest);
        }
        if input.take_presses() > 0 {
            frontend.app_mut().haptic_tick();
        }
        for (x, y) in input.take_taps() {
            frontend.app_mut().tap(x, y);
        }
        frontend.render(&mut g.compositor, window);
        if input.overlay_visible() {
            if captions_for.0 != input.generation() {
                g.overlay.clear_labels();
                captions.clear();
                for zone in input.zones() {
                    let Some((text, rect)) = crate::touch::caption(zone) else {
                        continue;
                    };
                    let (w, h) = (rect[2].ceil() as u32, rect[3].ceil() as u32);
                    if w == 0 || h == 0 {
                        continue;
                    }
                    let id = g
                        .overlay
                        .add_label(w, h, &crate::touch::caption_bitmap(text, w, h));
                    captions.push((id, rect, matches!(zone, crate::touch::Zone::Circle { .. })));
                }
                captions_for = (input.generation(), true);
            }
            let mut shapes = crate::touch::shapes(input.zones(), &input.pressed(), input.dim());
            shapes.extend(captions.iter().map(|&(id, rect, _)| slot_gfx::Shape {
                rect,
                round: false,
                rgba: [1.0, 1.0, 1.0, 0.9],
                label: Some(id),
            }));
            g.overlay.draw(window, &shapes);
        }
        let swap = Instant::now();
        let swapped = g.surface.swap();
        crate::latency::swapped(swap.elapsed().as_secs_f64() * 1000.0);
        if let Err(e) = swapped {
            log::error!("swap: {e}");
            g.surface.detach_window();
            shown = false;
            continue;
        }
        frontend.advance(&mut input);
        if frontend.restarting() {
            frontend.restart();
        }
        if frontend.powering_off() {
            frontend.poweroff();
            quit = true;
        }
        if let Some(left) = FRAME.checked_sub(began.elapsed()) {
            std::thread::sleep(left);
        }
    }
}
