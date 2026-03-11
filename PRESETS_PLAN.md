# TextStep Preset System — Implementation Plan

## Overview

Two types of presets, inspired by mpump's genre-based organization:

1. **Sound Presets** — Parameter snapshots for individual drum voices and the synth voice
2. **Pattern Presets** — Pre-made step sequences (genre-organized, like mpump)

---

## 1. Sound Presets

### What Gets Saved

**Drum Sound Preset** (per voice type):
- All 10 params: tune, sweep, color, snap, filter, drive, decay, volume, send_reverb, send_delay
- Tagged with voice type (Kick, Snare, etc.) so presets are voice-specific

**Synth Sound Preset**:
- All synth params: osc1/osc2 waveform+tune+level, sub level, filter cutoff+resonance+env, ADSR×3
- Optionally: effect send levels

**Drum Kit Preset** (full kit = 8 voices together):
- All 8 drum voice params bundled as one preset
- Named (e.g., "808 Classic", "Industrial", "Lo-Fi")

### Organization

```
presets/
  drum_sounds/
    kick/       — Kick-specific sound presets
    snare/      — Snare-specific sound presets
    ...
  synth_sounds/ — Synth voice presets
  kits/         — Full 8-voice drum kit presets
```

### Categories (inspired by mpump's genre model)

Sound presets tagged by style:
- **Drum kits:** 808, 909, Acoustic, Lo-Fi, Industrial, Minimal, Breakbeat, DnB, Ambient, Noise
- **Synth sounds:** Bass, Lead, Pad, Stab, Pluck, Acid, Drone, FX, Arp, Key

### Data Format

Reuse existing serde structures. Each preset is a small JSON file:

```json
{
  "name": "Deep 808",
  "category": "808",
  "description": "Low, boomy 808 kick with long decay",
  "voice_type": "Kick",
  "params": {
    "tune": 0.25,
    "sweep": 0.6,
    "color": 0.3,
    "snap": 0.15,
    "filter": 0.4,
    "drive": 0.2,
    "decay": 0.8,
    "volume": 0.85,
    "send_reverb": 0.1,
    "send_delay": 0.0
  }
}
```

### Implementation Steps

1. **PresetEntry struct** — name, category, description, voice_type, params (serde)
2. **Factory presets** — Hardcode ~50-80 sound presets as static data in Rust (no external files needed at first)
3. **Preset browser UI** — New view/modal: list presets filtered by current voice type, preview on highlight
4. **Load/Save keybindings**:
   - Browse presets: `p` (opens preset browser for selected track)
   - Save as preset: `Shift+P` (saves current track params as user preset)
5. **User presets** — Saved to `~/Library/Application Support/textstep/presets/` as JSON
6. **Preview** — Trigger the voice with preset params when highlighting in browser (audition)

---

## 2. Pattern Presets

### What Gets Saved

- 8×32 step grid (same as current pattern format)
- Optionally bundled with a kit preset (pattern + sounds together)
- Genre tag + name + description (like mpump's catalog)

### Genre Categories (borrowed from mpump)

techno, house, acid, dub-techno, trance, drum-and-bass, breakbeat, jungle,
garage, electro, idm, ambient, downtempo, glitch, industrial

### Data Format

Reuse existing hex-encoded step format from project.rs:

```json
{
  "name": "Four on the Floor",
  "genre": "techno",
  "description": "Classic techno kick pattern with offbeat hats",
  "steps": {
    "kick":   "88888888",
    "snare":  "00800080",
    "chh":    "44444444",
    "ohh":    "00000000",
    "ride":   "00000000",
    "clap":   "00800080",
    "cowbell": "00000000",
    "tom":    "00000000"
  }
}
```

### Implementation Steps

1. **PatternPreset struct** — name, genre, description, steps HashMap
2. **Factory patterns** — ~100 patterns (10 per genre, like mpump) hardcoded in Rust
3. **Pattern browser UI** — Genre list → pattern list (two-level like mpump's Picker)
4. **Load keybinding**: `Ctrl+P` opens pattern preset browser
5. **Merge modes**:
   - Replace: overwrites current pattern
   - Layer: OR-merge steps onto current pattern (non-destructive)

---

## 3. Combined "Scene" Presets (Future)

A scene = kit + pattern + effect settings. This is essentially what a Project slot is today,
but exposing it as a browsable preset library would let users quickly load full configurations.

Not needed for v1 — the kit and pattern presets cover the core use case.

---

## 4. UI Flow

### Preset Browser (TUI popup)

```
┌─── Drum Sound Presets: Kick ──────────┐
│                                        │
│  [808]                                 │
│    > Deep 808         Low, boomy...    │
│      Punchy 808       Tight attack...  │
│      Sub 808          Pure sub bass... │
│                                        │
│  [909]                                 │
│      Hard 909         Classic analog.. │
│      Soft 909         Rounded, warm..  │
│                                        │
│  [User]                                │
│      My Custom Kick   (saved 3/10)     │
│                                        │
│  ↑↓ Navigate  Enter Load  Esc Close   │
└────────────────────────────────────────┘
```

### Key Bindings Summary

| Key | Action |
|-----|--------|
| `p` | Open sound preset browser for selected track |
| `P` | Save current track as user sound preset |
| `Ctrl+P` | Open pattern preset browser |
| `↑/↓` | Navigate presets |
| `←/→` | Navigate categories |
| `Enter` | Load preset |
| `Space` | Preview/audition preset |
| `Esc` | Close browser |

---

## 5. Implementation Order

### Phase 1: Sound Presets (Core)
1. Define `SoundPreset` and `KitPreset` structs with serde
2. Create factory preset data (~10 per voice type = ~80 drum + ~20 synth)
3. Preset browser UI component
4. Wire `p` / `P` keybindings
5. User preset save/load to disk

### Phase 2: Pattern Presets
6. Define `PatternPreset` struct
7. Create factory patterns (~10 per genre × 15 genres = ~150)
8. Pattern browser UI (genre → pattern two-level)
9. Wire `Ctrl+P` keybinding
10. Replace/Layer merge modes

### Phase 3: Polish
11. Preview/audition while browsing
12. Preset search/filter
13. Import/export presets as files
14. Kit presets (full 8-voice bundles)

---

## Learnings from mpump

- **Genre-based organization** works well for discoverability (15 genres × 10 each)
- **Name + description** for every preset is essential for browsing
- **Separate catalogs per instrument type** (mpump has S-1, T-8, J-6; we have per-voice-type)
- **User presets in a separate section** (mpump's "extras" genre)
- **Simple data format** (JSON) enables easy sharing and editing
