# Drum Kit Presets — Parameter Reference

8 genre-specific kits for TextStep. Each kit defines all 8 voices.
Parameters: tune, sweep, color, snap, filter, drive, decay, volume (all 0.0–1.0).

Format: `ds(tune, sweep, color, snap, filter, drive, decay, volume)`

---

## Kit 1: 808 — Roland TR-808 Style

Character: Deep, boomy, analog warmth. Long decays on kick, crispy analog noise hats, snappy snare with tonal body.

| Voice    | tune | sweep | color | snap | filter | drive | decay | volume | Notes |
|----------|------|-------|-------|------|--------|-------|-------|--------|-------|
| Kick     | 0.20 | 0.70  | 0.15  | 0.10 | 0.50   | 0.10  | 0.80  | 0.85   | Deep sub, big pitch sweep, long decay |
| Snare    | 0.35 | 0.15  | 0.50  | 0.40 | 0.50   | 0.10  | 0.40  | 0.80   | Balanced tone/noise mix, moderate snap |
| CHH      | 0.60 | 0.00  | 0.50  | 0.40 | 0.65   | 0.00  | 0.08  | 0.70   | Tight, clean, analog noise character |
| OHH      | 0.50 | 0.00  | 0.50  | 0.30 | 0.55   | 0.00  | 0.50  | 0.65   | Same character as CHH, longer decay |
| Ride     | 0.55 | 0.00  | 0.45  | 0.15 | 0.50   | 0.00  | 0.70  | 0.55   | Metallic shimmer, long ring |
| Clap     | 0.50 | 0.30  | 0.50  | 0.50 | 0.50   | 0.10  | 0.40  | 0.70   | Classic layered flutter, moderate body |
| Cowbell  | 0.50 | 0.30  | 0.50  | 0.20 | 0.50   | 0.10  | 0.40  | 0.70   | Iconic dual-osc metallic tone |
| Tom      | 0.30 | 0.60  | 0.10  | 0.30 | 0.70   | 0.10  | 0.55  | 0.80   | Deep, pure tone, big sweep |

Key relationships: Low tune + high sweep on kick/tom. Clean drive across kit. Moderate filter. Color stays centered (balanced tone/noise).

---

## Kit 2: 909 — Roland TR-909 Style

Character: Punchy, crisp, harder-hitting than 808. Kick has more attack, snare is crackly with more noise. Hats are brighter and more metallic.

| Voice    | tune | sweep | color | snap | filter | drive | decay | volume | Notes |
|----------|------|-------|-------|------|--------|-------|-------|--------|-------|
| Kick     | 0.35 | 0.50  | 0.30  | 0.60 | 0.80   | 0.30  | 0.45  | 0.85   | Higher pitch, more snap/attack, shorter than 808 |
| Snare    | 0.45 | 0.10  | 0.60  | 0.60 | 0.65   | 0.20  | 0.35  | 0.85   | More noise (color up), crispier, bright filter |
| CHH      | 0.55 | 0.00  | 0.55  | 0.35 | 0.70   | 0.10  | 0.12  | 0.70   | Slightly metallic, more sizzle than 808 |
| OHH      | 0.55 | 0.00  | 0.55  | 0.35 | 0.65   | 0.10  | 0.55  | 0.70   | Brighter, more presence than 808 |
| Ride     | 0.60 | 0.00  | 0.50  | 0.25 | 0.60   | 0.10  | 0.65  | 0.60   | Cleaner metallic ring |
| Clap     | 0.55 | 0.25  | 0.55  | 0.60 | 0.60   | 0.15  | 0.35  | 0.75   | Tighter, snappier than 808 |
| Cowbell  | 0.55 | 0.25  | 0.45  | 0.30 | 0.60   | 0.10  | 0.35  | 0.65   | Brighter, slightly shorter |
| Tom      | 0.50 | 0.70  | 0.20  | 0.50 | 0.70   | 0.20  | 0.45  | 0.75   | More aggressive sweep, punchier |

Key relationships: Higher snap across the board vs 808. Filter pushed brighter. Slightly more drive/color. Shorter decays (tighter).

---

## Kit 3: Techno — Dark, Driving, Industrial-Leaning

Character: Relentless, hypnotic. Very deep kick with long tail, minimal snare (often just a clap or rimshot), driving hats, industrial textures.

| Voice    | tune | sweep | color | snap | filter | drive | decay | volume | Notes |
|----------|------|-------|-------|------|--------|-------|-------|--------|-------|
| Kick     | 0.22 | 0.75  | 0.25  | 0.55 | 0.60   | 0.35  | 0.70  | 0.90   | Deep, driving, moderate click, some grit |
| Snare    | 0.38 | 0.10  | 0.70  | 0.65 | 0.55   | 0.35  | 0.25  | 0.75   | Noisy, short, industrial-ish |
| CHH      | 0.50 | 0.05  | 0.60  | 0.50 | 0.55   | 0.25  | 0.06  | 0.65   | Dark, gritty, tight |
| OHH      | 0.45 | 0.10  | 0.65  | 0.40 | 0.50   | 0.30  | 0.40  | 0.60   | Dark, washy, driven |
| Ride     | 0.50 | 0.05  | 0.60  | 0.30 | 0.45   | 0.30  | 0.55  | 0.50   | Dark metallic, some grit |
| Clap     | 0.48 | 0.20  | 0.60  | 0.70 | 0.55   | 0.30  | 0.30  | 0.75   | Hard, snappy, driving |
| Cowbell  | 0.55 | 0.35  | 0.55  | 0.40 | 0.50   | 0.35  | 0.25  | 0.60   | Dark, percussive accent |
| Tom      | 0.35 | 0.75  | 0.30  | 0.55 | 0.55   | 0.35  | 0.50  | 0.75   | Deep, sweepy, industrial |

Key relationships: Filter pulled down (darker). Drive pushed up on everything (0.25–0.35). Color higher (more noise/grit). Kick dominates the mix (volume 0.90).

---

## Kit 4: House — Classic House, Warm and Bouncy

Character: Warm, round, musical. Bouncy kick with moderate punch, open snare/clap with room, shuffling hats, inviting and danceable.

| Voice    | tune | sweep | color | snap | filter | drive | decay | volume | Notes |
|----------|------|-------|-------|------|--------|-------|-------|--------|-------|
| Kick     | 0.28 | 0.60  | 0.20  | 0.45 | 0.65   | 0.15  | 0.60  | 0.85   | Warm, round, bouncy — between 808 and 909 |
| Snare    | 0.42 | 0.10  | 0.55  | 0.50 | 0.60   | 0.15  | 0.38  | 0.78   | Musical, balanced, warm |
| CHH      | 0.58 | 0.00  | 0.45  | 0.35 | 0.65   | 0.05  | 0.10  | 0.68   | Clean, shuffling, moderate brightness |
| OHH      | 0.52 | 0.00  | 0.45  | 0.30 | 0.60   | 0.05  | 0.50  | 0.65   | Open, airy, musical |
| Ride     | 0.58 | 0.00  | 0.40  | 0.20 | 0.55   | 0.05  | 0.65  | 0.55   | Warm shimmer, bell-like |
| Clap     | 0.52 | 0.30  | 0.50  | 0.55 | 0.55   | 0.10  | 0.42  | 0.72   | Classic house clap, some room |
| Cowbell  | 0.55 | 0.25  | 0.40  | 0.25 | 0.55   | 0.05  | 0.35  | 0.60   | Funky accent, warm |
| Tom      | 0.45 | 0.55  | 0.15  | 0.40 | 0.75   | 0.10  | 0.50  | 0.75   | Warm, musical toms |

Key relationships: Low drive everywhere (clean, warm). Filter slightly open (not too bright, not dark). Moderate everything — house lives in the sweet spot. Balanced volumes, kick leads.

---

## Kit 5: Minimal — Clean, Sparse, Subtle

Character: Reduced, precise. Every sound is intentionally small and controlled. Click kick, ghost snare, thin hats, barely-there percussion.

| Voice    | tune | sweep | color | snap | filter | drive | decay | volume | Notes |
|----------|------|-------|-------|------|--------|-------|-------|--------|-------|
| Kick     | 0.38 | 0.30  | 0.10  | 0.75 | 0.65   | 0.00  | 0.20  | 0.75   | Click-like, short, clean, mostly attack |
| Snare    | 0.50 | 0.00  | 0.35  | 0.70 | 0.75   | 0.00  | 0.15  | 0.65   | Crisp click/rimshot, very short |
| CHH      | 0.72 | 0.00  | 0.30  | 0.25 | 0.85   | 0.00  | 0.04  | 0.55   | Thin, bright, tiny |
| OHH      | 0.65 | 0.00  | 0.30  | 0.20 | 0.60   | 0.00  | 0.25  | 0.50   | Short even for an open hat |
| Ride     | 0.65 | 0.00  | 0.25  | 0.15 | 0.55   | 0.00  | 0.45  | 0.45   | Subtle ping, restrained |
| Clap     | 0.58 | 0.10  | 0.30  | 0.80 | 0.78   | 0.00  | 0.10  | 0.60   | Snap/click, barely a clap |
| Cowbell  | 0.52 | 0.10  | 0.25  | 0.30 | 0.50   | 0.00  | 0.18  | 0.50   | Muted, subtle accent |
| Tom      | 0.60 | 0.20  | 0.05  | 0.50 | 0.60   | 0.00  | 0.22  | 0.60   | Clean bleep, pure tone |

Key relationships: Drive at 0.00 everywhere (pristine). Short decays across the board. High snap (transient-focused). Low color (pure, not noisy). Lower volumes (space in the mix). High tune (smaller sounds).

---

## Kit 6: Lo-Fi — Gritty, Crushed, Vintage

Character: Sounds like it's been run through a worn tape machine and a cheap sampler. Rounded, crunchy, dark, with character in the imperfections.

| Voice    | tune | sweep | color | snap | filter | drive | decay | volume | Notes |
|----------|------|-------|-------|------|--------|-------|-------|--------|-------|
| Kick     | 0.28 | 0.55  | 0.45  | 0.30 | 0.40   | 0.45  | 0.55  | 0.78   | Warm, muddy, saturated, rounded attack |
| Snare    | 0.38 | 0.12  | 0.65  | 0.35 | 0.40   | 0.45  | 0.42  | 0.72   | Crunchy, noisy, dark, lo-fi character |
| CHH      | 0.48 | 0.00  | 0.60  | 0.25 | 0.38   | 0.35  | 0.10  | 0.62   | Dark, crushed, dull |
| OHH      | 0.42 | 0.00  | 0.55  | 0.20 | 0.35   | 0.35  | 0.50  | 0.60   | Washy, dark, saturated |
| Ride     | 0.42 | 0.00  | 0.55  | 0.12 | 0.35   | 0.25  | 0.75  | 0.52   | Dark, long, muffled |
| Clap     | 0.45 | 0.25  | 0.60  | 0.40 | 0.42   | 0.40  | 0.40  | 0.68   | Crunchy, smeared transient |
| Cowbell  | 0.48 | 0.30  | 0.55  | 0.20 | 0.40   | 0.35  | 0.38  | 0.58   | Dull, warm, vintage |
| Tom      | 0.38 | 0.50  | 0.35  | 0.25 | 0.45   | 0.40  | 0.52  | 0.70   | Warm, round, tape-saturated |

Key relationships: Filter LOW everywhere (0.35–0.45 = dark). Drive HIGH everywhere (0.25–0.45 = saturated). Color pushed up (noisy, characterful). Snap pulled down (softened transients). Tune pulled slightly lower.

---

## Kit 7: Electro — Bright, Aggressive, Funk-Influenced

Character: Inspired by Kraftwerk, Egyptian Lover, Drexciya. Sharp, zappy, futuristic. Big pitch sweeps, bright filter, aggressive transients.

| Voice    | tune | sweep | color | snap | filter | drive | decay | volume | Notes |
|----------|------|-------|-------|------|--------|-------|-------|--------|-------|
| Kick     | 0.30 | 0.80  | 0.25  | 0.70 | 0.85   | 0.25  | 0.50  | 0.88   | Zappy, big sweep, hard attack, bright |
| Snare    | 0.48 | 0.15  | 0.55  | 0.75 | 0.80   | 0.25  | 0.30  | 0.82   | Crack, aggressive, cutting |
| CHH      | 0.65 | 0.05  | 0.45  | 0.55 | 0.80   | 0.15  | 0.07  | 0.72   | Bright, sharp, precise |
| OHH      | 0.60 | 0.05  | 0.50  | 0.45 | 0.75   | 0.15  | 0.45  | 0.68   | Bright, sizzly, energetic |
| Ride     | 0.65 | 0.05  | 0.45  | 0.35 | 0.70   | 0.15  | 0.55  | 0.58   | Bright bell, metallic ping |
| Clap     | 0.55 | 0.20  | 0.50  | 0.70 | 0.75   | 0.20  | 0.28  | 0.78   | Sharp, funky, snappy |
| Cowbell  | 0.60 | 0.30  | 0.40  | 0.35 | 0.70   | 0.15  | 0.30  | 0.68   | Bright, funky, classic electro accent |
| Tom      | 0.50 | 0.80  | 0.15  | 0.60 | 0.80   | 0.20  | 0.40  | 0.78   | Zappy, big sweep, laser-tom |

Key relationships: Filter HIGH (0.70–0.85 = bright and cutting). Sweep HIGH on kick/tom (zappy pitch drops). Snap HIGH (sharp transients). Drive moderate (some edge, not muddy). Everything sounds "forward" in the mix.

---

## Kit 8: Ambient — Soft, Washy, Atmospheric

Character: Sounds dissolve into space. Soft attacks, long decays, low volume, dark and diffuse. Designed to sit in reverb/delay.

| Voice    | tune | sweep | color | snap | filter | drive | decay | volume | Notes |
|----------|------|-------|-------|------|--------|-------|-------|--------|-------|
| Kick     | 0.25 | 0.40  | 0.20  | 0.10 | 0.40   | 0.05  | 0.70  | 0.65   | Soft thud, pillow-like, no click |
| Snare    | 0.35 | 0.05  | 0.60  | 0.12 | 0.35   | 0.00  | 0.55  | 0.55   | Breathy, noise-wash, gentle |
| CHH      | 0.55 | 0.00  | 0.55  | 0.10 | 0.40   | 0.00  | 0.15  | 0.45   | Soft breath, whisper |
| OHH      | 0.48 | 0.00  | 0.50  | 0.08 | 0.38   | 0.00  | 0.75  | 0.45   | Long wash, atmospheric |
| Ride     | 0.50 | 0.00  | 0.45  | 0.08 | 0.35   | 0.00  | 0.90  | 0.42   | Very long shimmer, soft attack |
| Clap     | 0.45 | 0.15  | 0.55  | 0.15 | 0.38   | 0.00  | 0.55  | 0.50   | Soft flutter, diffuse |
| Cowbell  | 0.48 | 0.20  | 0.40  | 0.10 | 0.40   | 0.00  | 0.50  | 0.45   | Muted bell, distant |
| Tom      | 0.40 | 0.45  | 0.15  | 0.12 | 0.45   | 0.00  | 0.65  | 0.58   | Soft, deep, melodic |

Key relationships: Snap VERY LOW everywhere (0.08–0.15, nearly no transient). Drive at 0.00 (pristine/gentle). Long decays. Filter LOW (dark, muffled). Low volumes across the board (leave headroom for effects). Designed to be used with high send_reverb (0.3–0.5) and send_delay (0.2–0.4).

---

## Comparative Summary: What Makes Each Kit Distinct

| Parameter | 808 | 909 | Techno | House | Minimal | Lo-Fi | Electro | Ambient |
|-----------|-----|-----|--------|-------|---------|-------|---------|---------|
| **tune**  | Low | Mid | Low | Low-Mid | High | Low-Mid | Mid | Low-Mid |
| **sweep** | High | Mid | High | Mid | Low | Mid | Very High | Low-Mid |
| **color** | Mid | Mid-Hi | High | Mid | Low | High | Mid | Mid |
| **snap**  | Low-Mid | High | Mid-Hi | Mid | Very High | Low | Very High | Very Low |
| **filter**| Mid | High | Mid | Mid | High | Low | Very High | Low |
| **drive** | Low | Low-Mid | Mid | Low | Zero | High | Low-Mid | Zero |
| **decay** | Long | Mid | Mid-Long | Mid | Short | Mid | Mid | Long |
| **volume**| High | High | High (kick) | Balanced | Low | Mid | High | Low |

### Genre Fingerprints (the 2-3 params that define each):
- **808**: Low tune + high sweep + long decay (subby, boomy)
- **909**: High snap + bright filter + moderate drive (punchy, crisp)
- **Techno**: High drive + low filter + high kick volume (dark, relentless)
- **House**: Everything moderate, low drive (warm, balanced, musical)
- **Minimal**: Zero drive + high snap + short decay (clean, precise, small)
- **Lo-Fi**: High drive + low filter + low snap (dark, crushed, softened)
- **Electro**: High sweep + high filter + high snap (bright, zappy, aggressive)
- **Ambient**: Very low snap + zero drive + long decay (soft, atmospheric, diffuse)

### Suggested Send Effects per Kit

| Kit     | send_reverb | send_delay | Notes |
|---------|-------------|------------|-------|
| 808     | 0.05–0.15   | 0.00–0.05  | Dry, let the sounds speak |
| 909     | 0.10–0.20   | 0.05–0.10  | Moderate, some space |
| Techno  | 0.10–0.20   | 0.10–0.20  | Delay on hats, reverb on clap |
| House   | 0.15–0.25   | 0.05–0.15  | Classic house reverb on clap/snare |
| Minimal | 0.05–0.15   | 0.10–0.25  | Sparse reverb, delay for space |
| Lo-Fi   | 0.15–0.25   | 0.10–0.20  | Room verb adds character |
| Electro | 0.08–0.15   | 0.10–0.20  | Tight verb, funky delays |
| Ambient | 0.30–0.50   | 0.20–0.40  | Drenched, sounds dissolve into FX |
