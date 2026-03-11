// Transport state, BPM, loop config

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
