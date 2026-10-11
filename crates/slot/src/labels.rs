//! Missing cartridge labels, fetched in the background (network, image decode and the SD
//! write all stay off the frame loop). A label that already exists is never replaced.
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

use slot_store::Cart;

pub struct Ready {
    pub rom: PathBuf,
    pub label: PathBuf,
}

pub struct Labels {
    ready: Receiver<Ready>,
    // Dropping the sender wakes an idle worker. In-flight HTTP is bounded by its timeout and
    // shutdown never joins a network request.
    _stop: Sender<()>,
}

impl Labels {
    pub fn spawn(root: PathBuf, carts: Vec<Cart>) -> Self {
        let (out, ready) = mpsc::sync_channel(4);
        let (stop, stopped) = mpsc::channel();
        if carts.iter().all(|c| c.label.is_some()) {
            return Self { ready, _stop: stop };
        }
        let spawned = thread::Builder::new()
            .name("slot-labels".into())
            .spawn(move || {
                let mut downloader = slot_labels::Downloader::default();
                run(carts, stopped, out, |cart| downloader.prepare(&root, cart));
            });
        if let Err(e) = spawned {
            eprintln!("slot: labels: worker failed to start: {e}");
        }
        Self { ready, _stop: stop }
    }

    pub fn take(&self) -> Option<Ready> {
        self.ready.try_recv().ok()
    }
}

fn run(
    carts: Vec<Cart>,
    stop: Receiver<()>,
    out: SyncSender<Ready>,
    prepare: impl FnMut(&Cart) -> Result<PathBuf, slot_labels::Error>,
) {
    run_with_timing(
        carts,
        stop,
        out,
        prepare,
        [Duration::from_secs(30), Duration::from_secs(120)],
        Duration::from_secs(1),
    );
}

fn run_with_timing(
    carts: Vec<Cart>,
    stop: Receiver<()>,
    out: SyncSender<Ready>,
    mut prepare: impl FnMut(&Cart) -> Result<PathBuf, slot_labels::Error>,
    delays: [Duration; 2],
    pace: Duration,
) {
    let now = Instant::now();
    let mut jobs: VecDeque<_> = carts
        .into_iter()
        .filter(|c| c.label.is_none())
        .map(|c| (c, now, 0u8))
        .collect();
    let mut network_retry = now;
    let mut network_failures = 0u8;
    while !jobs.is_empty() {
        let now = Instant::now();
        let Some(index) = jobs
            .iter()
            .position(|(_, due, _)| *due <= now && network_retry <= now)
        else {
            if stop.recv_timeout(pace) != Err(mpsc::RecvTimeoutError::Timeout) {
                return;
            }
            continue;
        };
        if stop.try_recv() != Err(mpsc::TryRecvError::Empty) {
            return;
        }
        let (cart, _, attempts) = jobs.remove(index).unwrap();
        if !cart.rom.is_file() {
            continue;
        }
        match prepare(&cart) {
            Ok(label) => {
                network_failures = 0;
                if out
                    .send(Ready {
                        rom: cart.rom.clone(),
                        label,
                    })
                    .is_err()
                {
                    return;
                }
            }
            Err(e) => {
                eprintln!("slot: labels: {}: {e}", cart.stem);
                if matches!(e, slot_labels::Error::Network(_)) {
                    network_failures += 1;
                    if network_failures >= 3 {
                        eprintln!("slot: labels: three network failures; deferring the rest until next launch");
                        return;
                    }
                    network_retry = Instant::now() + delays[usize::from(network_failures - 1)];
                } else {
                    network_failures = 0;
                }
                if !matches!(
                    e,
                    slot_labels::Error::Unavailable(_) | slot_labels::Error::Image(_)
                ) {
                    if let Some(delay) = delays.get(usize::from(attempts)) {
                        jobs.push_back((cart, Instant::now() + *delay, attempts + 1));
                    }
                }
            }
        }
        // Pace successful transfers and unsupported games as well as failures.
        if stop.recv_timeout(pace) != Err(mpsc::RecvTimeoutError::Timeout) {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slot_store::Platform;

    fn cart(dir: &std::path::Path, name: &str) -> Cart {
        let rom = dir.join(format!("{name}.gba"));
        std::fs::write(&rom, []).unwrap();
        Cart {
            platform: Platform::Gba,
            stem: name.into(),
            rom,
            label: None,
            title: "".into(),
            code: "".into(),
            shell: None,
        }
    }

    #[test]
    fn labels_are_prepared_at_the_size_of_the_well_they_are_drawn_in() {
        assert_eq!(
            slot_labels::label_size(Platform::Gba),
            (slot_ui::LABEL_W, slot_ui::LABEL_H)
        );
        for platform in [Platform::Gb, Platform::Gbc] {
            assert_eq!(
                slot_labels::label_size(platform),
                (slot_ui::GB_LABEL_W, slot_ui::GB_LABEL_H)
            );
        }
    }

    #[test]
    fn offline_worker_gives_up_after_three_network_failures() {
        let dir = tempfile::tempdir().unwrap();
        let carts: Vec<_> = (0..50).map(|i| cart(dir.path(), &format!("G{i}"))).collect();
        let (_stop, stopped) = mpsc::channel();
        let (out, _ready) = mpsc::sync_channel(1);
        let mut calls = 0;
        run_with_timing(
            carts,
            stopped,
            out,
            |_| {
                calls += 1;
                Err(slot_labels::Error::Network("offline".into()))
            },
            [Duration::ZERO; 2],
            Duration::ZERO,
        );
        assert_eq!(calls, 3);
    }

    #[test]
    fn carts_with_a_label_are_never_requested_and_unavailable_ones_do_not_block() {
        let dir = tempfile::tempdir().unwrap();
        let mut own = cart(dir.path(), "Own");
        own.label = Some(dir.path().join("Own.png"));
        let carts = vec![own, cart(dir.path(), "Bad"), cart(dir.path(), "Good")];
        let (stop, stopped) = mpsc::channel();
        let (out, ready) = mpsc::sync_channel(4);
        let worker = thread::spawn(move || {
            run_with_timing(
                carts,
                stopped,
                out,
                |c| match c.stem.as_str() {
                    "Bad" => Err(slot_labels::Error::Unavailable("no art".into())),
                    "Good" => Ok(c.rom.with_extension("png")),
                    other => panic!("asked for {other}"),
                },
                [Duration::ZERO; 2],
                Duration::ZERO,
            )
        });
        let got = ready.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(got.rom.ends_with("Good.gba"));
        drop(stop);
        worker.join().unwrap();
    }
}
