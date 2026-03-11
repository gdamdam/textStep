// SequencerClock: sample-accurate step advancement
//
// This module is intentionally free of dependencies on other textstep modules.
// It uses only primitive types so it can be tested in complete isolation.

/// Emitted by the clock each time the sequencer advances to a new step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepEvent {
    /// Free-running step counter (never wraps).
    pub global_step: usize,
    /// Current beat number within a 4-beat bar (0..3).
    pub beat: u8,
    /// True when this step is the first step of a 16-step bar.
    pub is_bar_start: bool,
}

/// A sample-accurate clock that converts a stream of audio samples into
/// discrete sequencer steps at the tempo defined by BPM.
///
/// The clock is a pure "BPM to step pulse" converter — it does not know
/// about loop lengths or pattern boundaries. Callers are responsible for
/// mapping `global_step` into their own pattern/loop coordinates.
pub struct SequencerClock {
    samples_since_last_step: f64,
    current_step: usize,
    /// Tracks whether we still need to emit the very first step (step 0).
    first_step_pending: bool,
}

impl SequencerClock {
    /// Create a new clock starting at step 0. The first call to `advance`
    /// will immediately return `Some(StepEvent)` for step 0.
    pub fn new() -> Self {
        Self {
            samples_since_last_step: 0.0,
            current_step: 0,
            first_step_pending: true,
        }
    }

    /// Reset the clock back to step 0 and clear the sample accumulator.
    /// The next call to `advance` will immediately return step 0.
    pub fn reset(&mut self) {
        self.samples_since_last_step = 0.0;
        self.current_step = 0;
        self.first_step_pending = true;
    }

    /// Call once per audio sample. Returns `Some(StepEvent)` when the
    /// sequencer advances to a new step, `None` otherwise.
    ///
    /// - `bpm`: tempo in beats per minute (one beat = 4 sixteenth-note steps)
    /// - `sample_rate`: audio sample rate in Hz (e.g. 44100.0)
    /// - `swing`: 0.50 (straight) to 0.75 (heavy shuffle). Even steps are
    ///   lengthened, odd steps are shortened proportionally.
    pub fn advance(
        &mut self,
        bpm: f64,
        sample_rate: f64,
        swing: f32,
    ) -> Option<StepEvent> {
        // On the very first call after new() or reset(), emit step 0 immediately.
        if self.first_step_pending {
            self.first_step_pending = false;
            self.samples_since_last_step = 0.0;
            return Some(self.make_event());
        }

        let base_samples_per_step = sample_rate * 60.0 / bpm / 4.0;

        // Swing: even steps (0,2,4,...) are longer, odd steps (1,3,5,...) are shorter.
        // At swing=0.50 both are equal. At swing=0.75 even steps are 1.5x, odd are 0.5x.
        let swing64 = swing as f64;
        let samples_per_step = if self.current_step % 2 == 0 {
            base_samples_per_step * swing64 * 2.0
        } else {
            base_samples_per_step * (1.0 - swing64) * 2.0
        };

        self.samples_since_last_step += 1.0;

        if self.samples_since_last_step >= samples_per_step {
            self.samples_since_last_step -= samples_per_step;

            // Advance to next step (free-running, never wraps).
            self.current_step += 1;

            Some(self.make_event())
        } else {
            None
        }
    }

    /// Build a `StepEvent` from the current state.
    fn make_event(&self) -> StepEvent {
        StepEvent {
            global_step: self.current_step,
            beat: ((self.current_step / 4) % 4) as u8,
            is_bar_start: self.current_step % 16 == 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_call_returns_step_zero() {
        let mut clock = SequencerClock::new();
        let event = clock.advance(120.0, 44100.0, 0.5);
        assert!(event.is_some());
        let e = event.unwrap();
        assert_eq!(e.global_step, 0);
        assert_eq!(e.beat, 0);
        assert!(e.is_bar_start);
    }

    #[test]
    fn second_call_returns_none() {
        let mut clock = SequencerClock::new();
        let _ = clock.advance(120.0, 44100.0, 0.5); // step 0
        let event = clock.advance(120.0, 44100.0, 0.5);
        assert!(event.is_none());
    }

    #[test]
    fn advances_after_correct_number_of_samples() {
        let bpm = 120.0;
        let sr = 44100.0;
        let samples_per_step = sr * 60.0 / bpm / 4.0; // 5512.5

        let mut clock = SequencerClock::new();
        // First call: step 0 fires immediately.
        let e = clock.advance(bpm, sr, 0.5);
        assert_eq!(e.unwrap().global_step, 0);

        // Pump exactly samples_per_step samples — step 1 should fire.
        let count = samples_per_step.ceil() as usize;
        let mut step_events = Vec::new();
        for _ in 0..count {
            if let Some(e) = clock.advance(bpm, sr, 0.5) {
                step_events.push(e.global_step);
            }
        }
        assert_eq!(step_events, vec![1]);
    }

    #[test]
    fn global_step_never_wraps() {
        let bpm = 120.0;
        let sr = 44100.0;
        let samples_per_step = sr * 60.0 / bpm / 4.0;

        let mut clock = SequencerClock::new();
        let mut last_step = None;

        // Run through enough samples to hit steps 0..5 (6 events total).
        let total_samples = (samples_per_step * 5.0) as usize + 1;
        for _ in 0..total_samples {
            if let Some(e) = clock.advance(bpm, sr, 0.5) {
                last_step = Some(e.global_step);
            }
        }
        // Step 0 fires immediately, then 1..4 fire after accumulating samples.
        // 5 * samples_per_step samples covers steps 1-5, but step 0 already consumed
        // the first call, so we get steps 0, 1, 2, 3, 4.
        assert_eq!(last_step, Some(4));
    }

    #[test]
    fn beat_calculation() {
        // beat = (global_step / 4) % 4
        // Steps 0-3 -> beat 0, 4-7 -> beat 1, 8-11 -> beat 2, 12-15 -> beat 3
        let bpm = 120.0;
        let sr = 44100.0;
        let samples_per_step = sr * 60.0 / bpm / 4.0;

        let mut clock = SequencerClock::new();
        let mut beats = Vec::new();

        let total_samples = (samples_per_step * 16.0) as usize + 1;
        for _ in 0..total_samples {
            if let Some(e) = clock.advance(bpm, sr, 0.5) {
                beats.push(e.beat);
            }
        }
        // First 17 steps: 0..=16
        assert!(beats.len() >= 16);
        assert_eq!(beats[0], 0);  // step 0
        assert_eq!(beats[4], 1);  // step 4
        assert_eq!(beats[8], 2);  // step 8
        assert_eq!(beats[12], 3); // step 12
        assert_eq!(beats[16], 0); // step 16 -> (16/4)%4 = 0
    }

    #[test]
    fn is_bar_start_every_16_steps() {
        let bpm = 120.0;
        let sr = 44100.0;
        let samples_per_step = sr * 60.0 / bpm / 4.0;

        let mut clock = SequencerClock::new();
        let mut bar_starts = Vec::new();

        let total_samples = (samples_per_step * 33.0) as usize + 1;
        for _ in 0..total_samples {
            if let Some(e) = clock.advance(bpm, sr, 0.5) {
                if e.is_bar_start {
                    bar_starts.push(e.global_step);
                }
            }
        }
        // Steps 0, 16, and 32 are bar starts.
        assert_eq!(bar_starts, vec![0, 16, 32]);
    }

    #[test]
    fn reset_re_emits_step_zero() {
        let bpm = 120.0;
        let sr = 44100.0;
        let samples_per_step = sr * 60.0 / bpm / 4.0;

        let mut clock = SequencerClock::new();

        // Advance a few steps.
        let total = (samples_per_step * 5.0) as usize + 1;
        for _ in 0..total {
            clock.advance(bpm, sr, 0.5);
        }

        clock.reset();

        let event = clock.advance(bpm, sr, 0.5);
        assert!(event.is_some());
        let e = event.unwrap();
        assert_eq!(e.global_step, 0);
        assert!(e.is_bar_start);
    }
}
