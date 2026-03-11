// Minimal radix-2 FFT for spectrum analysis — all DSP from scratch.

use std::f32::consts::PI;

/// In-place radix-2 Cooley-Tukey FFT.
/// `re` and `im` must have length that is a power of 2.
pub fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(n, im.len());

    // Bit-reversal permutation
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    // Butterfly stages
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = -2.0 * PI / len as f32;
        for i in (0..n).step_by(len) {
            for k in 0..half {
                let w_re = (angle * k as f32).cos();
                let w_im = (angle * k as f32).sin();
                let t_re = re[i + k + half] * w_re - im[i + k + half] * w_im;
                let t_im = re[i + k + half] * w_im + im[i + k + half] * w_re;
                re[i + k + half] = re[i + k] - t_re;
                im[i + k + half] = im[i + k] - t_im;
                re[i + k] += t_re;
                im[i + k] += t_im;
            }
        }
        len <<= 1;
    }
}

/// Apply a Hann window in-place.
pub fn hann_window(samples: &mut [f32]) {
    let n = samples.len() as f32;
    for (i, s) in samples.iter_mut().enumerate() {
        let w = 0.5 * (1.0 - (2.0 * PI * i as f32 / n).cos());
        *s *= w;
    }
}

/// Compute magnitude spectrum from FFT output (first half only = positive frequencies).
/// Returns magnitudes in dB, normalized so 0 dB = full scale.
pub fn magnitude_db(re: &[f32], im: &[f32], out: &mut [f32]) {
    let n = re.len();
    let half = n / 2;
    let scale = 2.0 / n as f32; // FFT normalization
    for i in 0..out.len().min(half) {
        let mag = (re[i] * re[i] + im[i] * im[i]).sqrt() * scale;
        // Convert to dB with floor at -80 dB
        out[i] = (20.0 * mag.max(1e-4).log10()).max(-80.0);
    }
}

/// Map FFT bins to logarithmically-spaced frequency bands.
/// Uses RMS averaging (in linear magnitude, then back to dB) for smooth results,
/// especially in the bass where many bands share few bins.
pub fn bins_to_log_bands(
    magnitudes_db: &[f32],
    num_bands: usize,
    sample_rate: f32,
    freq_lo: f32,
    freq_hi: f32,
    fft_size: usize,
) -> Vec<f32> {
    let mut bands = vec![-80.0f32; num_bands];
    let log_lo = freq_lo.ln();
    let log_hi = freq_hi.ln();
    let bin_hz = sample_rate / fft_size as f32;

    for band in 0..num_bands {
        // Logarithmic frequency range for this band
        let t0 = band as f32 / num_bands as f32;
        let t1 = (band + 1) as f32 / num_bands as f32;
        let f0 = (log_lo + t0 * (log_hi - log_lo)).exp();
        let f1 = (log_lo + t1 * (log_hi - log_lo)).exp();

        let bin_start = (f0 / bin_hz).floor() as usize;
        let bin_end = (f1 / bin_hz).ceil() as usize;
        let bin_start = bin_start.max(1); // skip DC
        let bin_end = bin_end.min(magnitudes_db.len());

        if bin_start >= bin_end {
            // No bins in range — interpolate from neighbors
            let center_bin = ((f0 + f1) * 0.5 / bin_hz) as usize;
            let center_bin = center_bin.clamp(1, magnitudes_db.len() - 1);
            bands[band] = magnitudes_db[center_bin];
            continue;
        }

        // RMS in linear domain: convert dB back to linear, average, convert back
        let count = (bin_end - bin_start) as f32;
        let mut sum_sq: f32 = 0.0;
        for b in bin_start..bin_end {
            // dB to linear amplitude: 10^(dB/20)
            let lin = 10.0f32.powf(magnitudes_db[b] / 20.0);
            sum_sq += lin * lin;
        }
        let rms = (sum_sq / count).sqrt();
        bands[band] = (20.0 * rms.max(1e-4).log10()).max(-80.0);
    }
    bands
}
