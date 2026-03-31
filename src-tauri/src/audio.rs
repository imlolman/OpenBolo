use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    Arc,
};

use crate::config::SAMPLE_RATE;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
}

pub struct AudioRecorder {
    stream: Option<cpal::Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    rms: Arc<AtomicU32>,
    recording: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
    native_rate: u32,
    native_channels: u16,
}

unsafe impl Send for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            rms: Arc::new(AtomicU32::new(0)),
            recording: Arc::new(AtomicBool::new(false)),
            generation: Arc::new(AtomicU64::new(0)),
            native_rate: SAMPLE_RATE,
            native_channels: 1,
        }
    }

    pub fn start(&mut self, device_name: Option<&str>) -> anyhow::Result<()> {
        // Kill any lingering old stream first
        self.recording.store(false, Ordering::SeqCst);
        if self.stream.is_some() {
            self.stream = None;
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let host = cpal::default_host();
        let device = if let Some(name) = device_name {
            host.input_devices()?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| anyhow::anyhow!("Device '{}' not found", name))?
        } else {
            host.default_input_device()
                .ok_or_else(|| anyhow::anyhow!("No default input device"))?
        };

        let default_config = device.default_input_config()?;
        let native_rate = default_config.sample_rate().0;
        let native_channels = default_config.channels();
        self.native_rate = native_rate;
        self.native_channels = native_channels;

        let config = cpal::StreamConfig {
            channels: native_channels,
            sample_rate: cpal::SampleRate(native_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Bump generation so any old callbacks become invalid
        let gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;

        self.buffer.lock().clear();
        self.recording.store(true, Ordering::SeqCst);

        let buf = Arc::clone(&self.buffer);
        let rms = Arc::clone(&self.rms);
        let generation = Arc::clone(&self.generation);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if generation.load(Ordering::Relaxed) != gen {
                    return;
                }
                buf.lock().extend_from_slice(data);
                let sum: f32 = data.iter().map(|s| s * s).sum();
                let val = (sum / data.len() as f32).sqrt();
                rms.store(val.to_bits(), Ordering::Relaxed);
            },
            |err| {
                eprintln!("[audio] stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> Vec<f32> {
        self.recording.store(false, Ordering::SeqCst);
        // Invalidate this stream's generation so its callbacks can't write anymore
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.stream = None;
        // Small delay to let any in-flight callbacks finish
        std::thread::sleep(std::time::Duration::from_millis(20));

        let raw = std::mem::take(&mut *self.buffer.lock());

        let mono = if self.native_channels > 1 {
            let ch = self.native_channels as usize;
            raw.chunks(ch)
                .map(|frame| frame.iter().sum::<f32>() / ch as f32)
                .collect::<Vec<f32>>()
        } else {
            raw
        };

        let trimmed = trim_silence(&mono, 0.01);
        if trimmed.is_empty() {
            return Vec::new();
        }

        let mut resampled = if self.native_rate == SAMPLE_RATE {
            trimmed.to_vec()
        } else {
            resample(trimmed, self.native_rate, SAMPLE_RATE)
        };

        // Normalize: scale so peak reaches ~0.8 — cap gain at 5x to avoid amplifying noise
        let trimmed_peak = resampled.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if trimmed_peak > 1e-4 {
            let gain = (0.8 / trimmed_peak).min(5.0);
            if gain > 1.2 {
                for s in resampled.iter_mut() {
                    *s *= gain;
                }
            }
        }

        resampled
    }

    pub fn get_rms(&self) -> f32 {
        f32::from_bits(self.rms.load(Ordering::Relaxed))
    }

    pub fn list_devices() -> Vec<DeviceInfo> {
        let host = cpal::default_host();
        let default_name = host
            .default_input_device()
            .and_then(|d| d.name().ok());

        host.input_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| {
                        let name = d.name().ok()?;
                        Some(DeviceInfo {
                            is_default: default_name.as_deref() == Some(&name),
                            name,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn test_mic(device_name: Option<&str>) -> anyhow::Result<f64> {
        let host = cpal::default_host();
        let device = if let Some(name) = device_name {
            host.input_devices()?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| anyhow::anyhow!("Device not found"))?
        } else {
            host.default_input_device()
                .ok_or_else(|| anyhow::anyhow!("No default input device"))?
        };

        let default_config = device.default_input_config()?;
        let config = cpal::StreamConfig {
            channels: default_config.channels(),
            sample_rate: default_config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let peak = Arc::new(AtomicU32::new(0f32.to_bits()));
        let peak_clone = Arc::clone(&peak);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let max = data.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                let cur = f32::from_bits(peak_clone.load(Ordering::Relaxed));
                if max > cur {
                    peak_clone.store(max.to_bits(), Ordering::Relaxed);
                }
            },
            |_| {},
            None,
        )?;

        stream.play()?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        drop(stream);

        Ok(f32::from_bits(peak.load(Ordering::Relaxed)) as f64)
    }
}

/// Trim leading and trailing silence from mono audio.
/// Uses a sliding window RMS check against the threshold.
fn trim_silence(samples: &[f32], threshold: f32) -> &[f32] {
    if samples.is_empty() {
        return samples;
    }
    let window = 480; // ~30ms at 16kHz, ~10ms at 48kHz
    let mut start = 0;
    let mut end = samples.len();

    // Find first non-silent window from start
    for i in (0..samples.len()).step_by(window / 2) {
        let chunk_end = (i + window).min(samples.len());
        let chunk = &samples[i..chunk_end];
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        if rms > threshold {
            start = i;
            break;
        }
    }

    // Find last non-silent window from end
    for i in (0..samples.len()).rev().step_by(window / 2) {
        let chunk_start = i.saturating_sub(window);
        let chunk = &samples[chunk_start..=i.min(samples.len() - 1)];
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        if rms > threshold {
            end = (i + 1).min(samples.len());
            break;
        }
    }

    if start >= end {
        return &[];
    }
    &samples[start..end]
}

fn resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || input.is_empty() {
        return input.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let int_ratio = ratio.round() as usize;

    if int_ratio > 1 && (ratio - int_ratio as f64).abs() < 0.01 {
        return input
            .chunks_exact(int_ratio)
            .map(|chunk| chunk.iter().sum::<f32>() / int_ratio as f32)
            .collect();
    }

    let filtered = lowpass_filter(input, to_rate as f64 / from_rate as f64 * 0.45);

    let out_len = (input.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;
        let sample = if idx + 1 < filtered.len() {
            filtered[idx] * (1.0 - frac) + filtered[idx + 1] * frac
        } else {
            filtered[idx.min(filtered.len() - 1)]
        };
        output.push(sample);
    }
    output
}

fn lowpass_filter(input: &[f32], cutoff_ratio: f64) -> Vec<f32> {
    let half_width: usize = 16;
    let kernel_len = half_width * 2 + 1;
    let mut kernel = vec![0.0f64; kernel_len];
    let mut sum = 0.0;
    for j in 0..kernel_len {
        let n = j as f64 - half_width as f64;
        let sinc = if n.abs() < 1e-10 {
            2.0 * std::f64::consts::PI * cutoff_ratio
        } else {
            (2.0 * std::f64::consts::PI * cutoff_ratio * n).sin() / n
        };
        let w = 0.42 - 0.5 * (2.0 * std::f64::consts::PI * j as f64 / (kernel_len - 1) as f64).cos()
            + 0.08 * (4.0 * std::f64::consts::PI * j as f64 / (kernel_len - 1) as f64).cos();
        kernel[j] = sinc * w;
        sum += kernel[j];
    }
    for k in kernel.iter_mut() {
        *k /= sum;
    }

    let len = input.len();
    let mut output = Vec::with_capacity(len);
    for i in 0..len {
        let mut val = 0.0f64;
        for j in 0..kernel_len {
            let idx = i as isize + j as isize - half_width as isize;
            let sample = if idx < 0 {
                input[0]
            } else if idx >= len as isize {
                input[len - 1]
            } else {
                input[idx as usize]
            };
            val += sample as f64 * kernel[j];
        }
        output.push(val as f32);
    }
    output
}
