// Lock-free shared buffer for audio visualization data.
// Audio thread writes, UI thread reads. No locks, no allocations.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// Number of samples stored in the waveform ring buffer.
/// ~85ms at 48kHz — enough for a 4096-point FFT window.
pub const WAVEFORM_SIZE: usize = 4096;

pub struct AudioDisplayBuffer {
    /// Peak level (f32 stored as bits). Updated by audio thread each buffer.
    peak: AtomicU32,
    /// Ring buffer of post-output audio samples (f32 stored as bits).
    waveform: [AtomicU32; WAVEFORM_SIZE],
    /// Current write position in the ring buffer.
    write_pos: AtomicUsize,
}

impl AudioDisplayBuffer {
    pub fn new() -> Self {
        Self {
            peak: AtomicU32::new(0),
            waveform: std::array::from_fn(|_| AtomicU32::new(0)),
            write_pos: AtomicUsize::new(0),
        }
    }

    /// Write a single sample into the ring buffer (called from audio thread).
    #[inline]
    pub fn push_sample(&self, sample: f32) {
        let pos = self.write_pos.load(Ordering::Relaxed);
        self.waveform[pos].store(sample.to_bits(), Ordering::Relaxed);
        self.write_pos.store((pos + 1) % WAVEFORM_SIZE, Ordering::Relaxed);
    }

    /// Update the peak level (called from audio thread, once per buffer).
    #[inline]
    pub fn set_peak(&self, peak: f32) {
        self.peak.store(peak.to_bits(), Ordering::Relaxed);
    }

    /// Read the current peak level (called from UI thread).
    pub fn get_peak(&self) -> f32 {
        f32::from_bits(self.peak.load(Ordering::Relaxed))
    }

    /// Read the latest N samples from the ring buffer for display.
    /// Returns samples in chronological order (oldest first).
    pub fn read_waveform(&self, out: &mut [f32]) {
        let pos = self.write_pos.load(Ordering::Relaxed);
        let n = out.len().min(WAVEFORM_SIZE);
        let start = (pos + WAVEFORM_SIZE - n) % WAVEFORM_SIZE;
        for i in 0..n {
            let idx = (start + i) % WAVEFORM_SIZE;
            out[i] = f32::from_bits(self.waveform[idx].load(Ordering::Relaxed));
        }
    }
}
