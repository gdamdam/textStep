#!/usr/bin/env python3
"""Convert mpump pattern JSON files to textstep Rust preset source files."""

import json
import os

MPUMP_DIR = os.path.join(os.path.dirname(__file__), '..', '..', 'mpump', 'mpump', 'server', 'public', 'data')
OUT_DIR = os.path.join(os.path.dirname(__file__), '..', 'src', 'presets')

# ── MIDI note → textstep drum track mapping ──────────────────────────────────
# Track order: Kick(0), Snare(1), CHH(2), OHH(3), Ride(4), Clap(5), Cowbell(6), Tom(7)
MIDI_TO_TRACK = {
    36: 0,  # Kick
    38: 1,  # Snare
    42: 2,  # CHH (Closed Hi-Hat)
    46: 3,  # OHH (Open Hi-Hat)
    51: 4,  # Ride Cymbal
    56: 4,  # Cowbell (GM) → Ride (closest match)
    50: 5,  # High Tom → Clap
    49: 5,  # Crash Cymbal → Clap
    37: 6,  # Side Stick / Rimshot → Cowbell
    47: 7,  # Low-Mid Tom → Tom
}

# Genre display names (mpump key → textstep display name)
GENRE_NAMES = {
    'techno': 'Techno',
    'acid-techno': 'Acid Techno',
    'trance': 'Trance',
    'dub-techno': 'Dub Techno',
    'idm': 'IDM',
    'edm': 'EDM',
    'drum-and-bass': 'Drum & Bass',
    'house': 'House',
    'breakbeat': 'Breakbeat',
    'jungle': 'Jungle',
    'garage': 'Garage',
    'ambient': 'Ambient',
    'glitch': 'Glitch',
    'electro': 'Electro',
    'downtempo': 'Downtempo',
}

def steps_to_hex(steps_32):
    """Convert 32-element bool list to 8-char hex string."""
    hex_str = ''
    for i in range(0, 32, 4):
        nibble = 0
        for j in range(4):
            if i + j < len(steps_32) and steps_32[i + j]:
                nibble |= (8 >> j)
        hex_str += format(nibble, 'x')
    return hex_str


def convert_drum_pattern(pattern_16):
    """Convert mpump 16-step drum pattern to 8 hex strings (tiled to 32 steps)."""
    tracks = [[False] * 32 for _ in range(8)]

    for step_idx, step_hits in enumerate(pattern_16):
        if not step_hits:
            continue
        for hit in step_hits:
            midi_note = hit['note']
            track = MIDI_TO_TRACK.get(midi_note)
            if track is not None:
                # Tile: step_idx maps to both step_idx and step_idx+16
                tracks[track][step_idx] = True
                tracks[track][step_idx + 16] = True

    return [steps_to_hex(t) for t in tracks]


def convert_synth_pattern(pattern_16, base_note=48):
    """Convert mpump 16-step melodic pattern to 32 (note, vel, len) tuples.

    mpump format: {semi, vel, slide} or null
    textstep format: (note: u8, velocity: u8, length: u8) where vel=0 means rest
    """
    steps = []
    for step in pattern_16:
        if step is None:
            steps.append((0, 0, 1))  # rest
        else:
            note = base_note + step['semi']
            note = max(0, min(127, note))
            # vel: 1.0 → 100, 1.3 → 120
            vel = int(step['vel'] * 100)
            vel = max(1, min(127, vel))
            steps.append((note, vel, 1))

    # Tile 16 → 32
    result = steps + steps
    return result


def generate_drum_presets():
    """Generate pattern_presets.rs from mpump T-8 drum patterns."""
    with open(os.path.join(MPUMP_DIR, 'patterns-t8-drums.json')) as f:
        data = json.load(f)

    lines = []
    lines.append('use crate::sequencer::drum_pattern::NUM_DRUM_TRACKS;')
    lines.append('')
    lines.append('pub struct PatternPreset {')
    lines.append('    pub name: &\'static str,')
    lines.append('    pub genre: &\'static str,')
    lines.append('    /// Hex-encoded steps per track (8 hex chars = 32 steps each)')
    lines.append('    pub steps: [&\'static str; NUM_DRUM_TRACKS],')
    lines.append('}')
    lines.append('')
    lines.append('// Track order: Kick, Snare, CHH, OHH, Ride, Clap, Cowbell, Tom')
    lines.append('')
    lines.append('pub static PATTERN_PRESETS: &[PatternPreset] = &[')

    for genre_key, patterns in data.items():
        genre_name = GENRE_NAMES.get(genre_key, genre_key.replace('-', ' ').title())
        lines.append(f'    // ── {genre_name} {"─" * (67 - len(genre_name))}')
        for i, pat in enumerate(patterns):
            hex_tracks = convert_drum_pattern(pat)
            name = f'{genre_name} {i + 1}'
            hex_str = '", "'.join(hex_tracks)
            lines.append(f'    PatternPreset {{ name: "{name}", genre: "{genre_name}",')
            lines.append(f'        steps: ["{hex_str}"] }},')

    lines.append('];')
    lines.append('')
    lines.append('pub fn genres() -> Vec<&\'static str> {')
    lines.append('    let mut g: Vec<&\'static str> = Vec::new();')
    lines.append('    for p in PATTERN_PRESETS {')
    lines.append('        if !g.contains(&p.genre) {')
    lines.append('            g.push(p.genre);')
    lines.append('        }')
    lines.append('    }')
    lines.append('    g')
    lines.append('}')
    lines.append('')
    lines.append('pub fn presets_for_genre(genre: &str) -> Vec<&\'static PatternPreset> {')
    lines.append('    PATTERN_PRESETS.iter().filter(|p| p.genre == genre).collect()')
    lines.append('}')
    lines.append('')

    return '\n'.join(lines)


def generate_synth_presets():
    """Generate synth_pattern_presets.rs from mpump S-1 and T-8 bass patterns."""
    with open(os.path.join(MPUMP_DIR, 'patterns-s1.json')) as f:
        s1_data = json.load(f)
    with open(os.path.join(MPUMP_DIR, 'patterns-t8-bass.json')) as f:
        bass_data = json.load(f)

    lines = []
    lines.append('use crate::sequencer::synth_pattern::MAX_STEPS;')
    lines.append('')
    lines.append('pub struct SynthPatternPreset {')
    lines.append('    pub name: &\'static str,')
    lines.append('    pub genre: &\'static str,')
    lines.append('    /// (note, velocity, length) tuples. velocity 0 = rest.')
    lines.append('    pub steps: [(u8, u8, u8); MAX_STEPS],')
    lines.append('}')
    lines.append('')
    lines.append('pub static SYNTH_PATTERN_PRESETS: &[SynthPatternPreset] = &[')

    # S-1 patterns (melodic, base note C3=48)
    for genre_key, patterns in s1_data.items():
        genre_name = GENRE_NAMES.get(genre_key, genre_key.replace('-', ' ').title())
        lines.append(f'    // ── {genre_name} (S-1) {"─" * (62 - len(genre_name))}')
        for i, pat in enumerate(patterns):
            steps = convert_synth_pattern(pat, base_note=48)
            name = f'{genre_name} {i + 1}'
            steps_str = ', '.join(f'({s[0]}, {s[1]}, {s[2]})' for s in steps)
            lines.append(f'    SynthPatternPreset {{ name: "{name}", genre: "{genre_name}",')
            lines.append(f'        steps: [{steps_str}] }},')

    # T-8 bass patterns (bass, base note C3=48)
    for genre_key, patterns in bass_data.items():
        genre_name = GENRE_NAMES.get(genre_key, genre_key.replace('-', ' ').title())
        bass_genre = f'{genre_name} Bass'
        lines.append(f'    // ── {bass_genre} {"─" * (67 - len(bass_genre))}')
        for i, pat in enumerate(patterns):
            steps = convert_synth_pattern(pat, base_note=36)  # bass = C2
            name = f'{bass_genre} {i + 1}'
            steps_str = ', '.join(f'({s[0]}, {s[1]}, {s[2]})' for s in steps)
            lines.append(f'    SynthPatternPreset {{ name: "{name}", genre: "{bass_genre}",')
            lines.append(f'        steps: [{steps_str}] }},')

    lines.append('];')
    lines.append('')
    lines.append('pub fn genres() -> Vec<&\'static str> {')
    lines.append('    let mut g: Vec<&\'static str> = Vec::new();')
    lines.append('    for p in SYNTH_PATTERN_PRESETS {')
    lines.append('        if !g.contains(&p.genre) {')
    lines.append('            g.push(p.genre);')
    lines.append('        }')
    lines.append('    }')
    lines.append('    g')
    lines.append('}')
    lines.append('')
    lines.append('pub fn presets_for_genre(genre: &str) -> Vec<&\'static SynthPatternPreset> {')
    lines.append('    SYNTH_PATTERN_PRESETS.iter().filter(|p| p.genre == genre).collect()')
    lines.append('}')
    lines.append('')

    return '\n'.join(lines)


if __name__ == '__main__':
    print("Converting mpump drum patterns...")
    drum_rs = generate_drum_presets()
    drum_path = os.path.join(OUT_DIR, 'pattern_presets.rs')
    with open(drum_path, 'w') as f:
        f.write(drum_rs)
    print(f"  Written: {drum_path}")

    print("Converting mpump synth patterns...")
    synth_rs = generate_synth_presets()
    synth_path = os.path.join(OUT_DIR, 'synth_pattern_presets.rs')
    with open(synth_path, 'w') as f:
        f.write(synth_rs)
    print(f"  Written: {synth_path}")

    print("Done!")
