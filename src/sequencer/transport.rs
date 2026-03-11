//! Transport state: play/pause/stop, BPM, loop configuration, swing amount.

/// Sequencer playback state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordMode {
    Off,
    On,
}

/// Per-section loop length settings (8/16/24/32 steps for drum and synth independently).
#[derive(Clone, Copy, Debug)]
pub struct LoopConfig {
    pub enabled: bool,
    pub drum_length: u8,  // 8, 16, 24, or 32
    pub synth_length: u8, // 8, 16, 24, or 32
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            drum_length: 32,
            synth_length: 32,
        }
    }
}

/// Master transport: play state, tempo (BPM), record mode, loop config, and swing.
#[derive(Clone, Copy, Debug)]
pub struct Transport {
    pub state: PlayState,
    pub bpm: f64, // 60.0..=300.0
    pub record_mode: RecordMode,
    pub loop_config: LoopConfig,
    pub swing: f32, // 0.50 (straight) .. 0.75 (heavy shuffle)
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            state: PlayState::Stopped,
            bpm: 120.0,
            record_mode: RecordMode::Off,
            loop_config: LoopConfig::default(),
            swing: 0.50,
        }
    }
}
