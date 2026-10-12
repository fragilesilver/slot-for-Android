use std::sync::Arc;

use ndk::audio::{
    AudioCallbackResult, AudioDirection, AudioFormat, AudioPerformanceMode, AudioSharingMode,
    AudioStream, AudioStreamBuilder,
};

use super::ring::Ring;
use super::sink::{AudioError, AudioSink};

/// AAudio output. The stream runs at the device's native rate and pulls from the shared ring.
pub struct AndroidAudio {
    ring: Arc<Ring>,
    stream: Option<Stream>,
}

struct Stream(AudioStream);

// AAudio streams are thread-safe by contract; the wrapper only exists because the ndk type
// holds raw pointers and boxed callbacks.
unsafe impl Send for Stream {}

impl AndroidAudio {
    pub fn new() -> Self {
        AndroidAudio {
            ring: Arc::new(Ring::new(0)),
            stream: None,
        }
    }
}

impl Default for AndroidAudio {
    fn default() -> Self {
        AndroidAudio::new()
    }
}

impl Drop for AndroidAudio {
    fn drop(&mut self) {
        if let Some(Stream(s)) = self.stream.take() {
            let _ = s.request_stop();
        }
    }
}

fn err(e: impl std::fmt::Display) -> AudioError {
    AudioError::Device(e.to_string())
}

impl AudioSink for AndroidAudio {
    fn open(&mut self, _sample_rate: u32) -> Result<(), AudioError> {
        if let Some(Stream(s)) = self.stream.take() {
            let _ = s.request_stop();
        }
        let ring = self.ring.clone();
        let feed = ring.clone();
        let stream = AudioStreamBuilder::new()
            .map_err(err)?
            .direction(AudioDirection::Output)
            .channel_count(2)
            .format(AudioFormat::PCM_I16)
            .performance_mode(AudioPerformanceMode::LowLatency)
            .sharing_mode(AudioSharingMode::Shared)
            .data_callback(Box::new(move |_, data, frames| {
                // SAFETY: AAudio hands us `frames` interleaved stereo i16 frames.
                let out = unsafe {
                    std::slice::from_raw_parts_mut(data as *mut i16, frames as usize * 2)
                };
                feed.fill(out);
                AudioCallbackResult::Continue
            }))
            .error_callback(Box::new(|_, e| eprintln!("slot: audio: {e}")))
            .open_stream()
            .map_err(err)?;
        ring.reopen(stream.sample_rate() as u32);
        // Two bursts of buffering keeps latency low without underruns on a busy core.
        let _ = stream.set_buffer_size_in_frames(stream.frames_per_burst() * 2);
        stream.request_start().map_err(err)?;
        self.stream = Some(Stream(stream));
        Ok(())
    }

    fn ring(&self) -> Arc<Ring> {
        self.ring.clone()
    }
}
