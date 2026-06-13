use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SizedSample, Stream};

pub const STREAM_INPUT_QUEUE_CAPACITY: usize = 4096;

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub sample_rate: u32,
    pub channels: usize,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InputDiagnostics {
    pub input_drops: u64,
    pub input_timeouts: u64,
    pub captured_frames: u64,
    pub queue_ms: u64,
    pub sample_rate: u64,
    pub channels: u64,
    pub input_peak_per_mille: u64,
    pub processed_peak_per_mille: u64,
    pub resampler_pending_ms: u64,
    pub encoded_queue_ms: u64,
    pub input_silence_ms: u64,
    pub input_restarts: u64,
}

#[derive(Default)]
struct InputStats {
    input_drops: AtomicU64,
    input_timeouts: AtomicU64,
    captured_frames: AtomicU64,
    queued_frames: AtomicU64,
    sample_rate: AtomicU64,
    channels: AtomicU64,
    input_peak_per_mille: AtomicU64,
}

pub struct StreamInput {
    rx: Receiver<QueuedAudioFrame>,
    _stream: Stream,
    stats: Arc<InputStats>,
}

impl StreamInput {
    pub fn open(device_name: Option<&str>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = get_input_device(&host, device_name)?;
        let supported_config = device
            .default_input_config()
            .map_err(|error| format!("Input config error: {error}"))?;
        let sample_format = supported_config.sample_format();
        let config = supported_config.config();
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0;
        let (tx, rx) = mpsc::sync_channel(STREAM_INPUT_QUEUE_CAPACITY);
        let stats = Arc::new(InputStats::default());
        stats
            .sample_rate
            .store(sample_rate as u64, Ordering::Relaxed);
        stats.channels.store(channels as u64, Ordering::Relaxed);
        let stream = match sample_format {
            SampleFormat::F32 => {
                build_input_stream::<f32>(&device, &config, tx, Arc::clone(&stats))
            }
            SampleFormat::I16 => {
                build_input_stream::<i16>(&device, &config, tx, Arc::clone(&stats))
            }
            SampleFormat::U16 => {
                build_input_stream::<u16>(&device, &config, tx, Arc::clone(&stats))
            }
            SampleFormat::I8 => build_input_stream::<i8>(&device, &config, tx, Arc::clone(&stats)),
            SampleFormat::I32 => {
                build_input_stream::<i32>(&device, &config, tx, Arc::clone(&stats))
            }
            SampleFormat::U8 => build_input_stream::<u8>(&device, &config, tx, Arc::clone(&stats)),
            SampleFormat::U32 => {
                build_input_stream::<u32>(&device, &config, tx, Arc::clone(&stats))
            }
            other => Err(format!("Unsupported input sample format: {other}")),
        }?;
        stream
            .play()
            .map_err(|error| format!("Input stream start error: {error}"))?;
        let _ = (channels, sample_rate);
        Ok(Self {
            rx,
            _stream: stream,
            stats,
        })
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<AudioFrame, mpsc::RecvTimeoutError> {
        let frame = self.rx.recv_timeout(timeout)?;
        self.stats
            .queued_frames
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(frame.frames as u64))
            })
            .ok();
        Ok(frame.frame)
    }

    pub fn note_timeout(&self) {
        self.stats.input_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn diagnostics(&self) -> InputDiagnostics {
        InputDiagnostics {
            input_drops: self.stats.input_drops.load(Ordering::Relaxed),
            input_timeouts: self.stats.input_timeouts.load(Ordering::Relaxed),
            captured_frames: self.stats.captured_frames.load(Ordering::Relaxed),
            queue_ms: frames_to_ms(
                self.stats.queued_frames.load(Ordering::Relaxed),
                self.stats.sample_rate.load(Ordering::Relaxed),
            ),
            sample_rate: self.stats.sample_rate.load(Ordering::Relaxed),
            channels: self.stats.channels.load(Ordering::Relaxed),
            input_peak_per_mille: self.stats.input_peak_per_mille.load(Ordering::Relaxed),
            processed_peak_per_mille: 0,
            resampler_pending_ms: 0,
            encoded_queue_ms: 0,
            input_silence_ms: 0,
            input_restarts: 0,
        }
    }
}

struct QueuedAudioFrame {
    frame: AudioFrame,
    frames: usize,
}

fn get_input_device(host: &cpal::Host, name: Option<&str>) -> Result<cpal::Device, String> {
    if let Some(name) = name.filter(|name| !name.is_empty()) {
        if let Ok(mut devices) = host.input_devices() {
            if let Some(device) = devices.find(|device| device.name().ok().as_deref() == Some(name))
            {
                return Ok(device);
            }
        }
        return Err(format!("Input device not found: {name}"));
    }
    host.default_input_device()
        .ok_or_else(|| String::from("No audio input device available"))
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    tx: SyncSender<QueuedAudioFrame>,
    stats: Arc<InputStats>,
) -> Result<Stream, String>
where
    T: SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    let sample_rate = config.sample_rate.0;
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let samples = data
                    .iter()
                    .map(|sample| sample.to_sample::<f32>())
                    .collect::<Vec<f32>>();
                let frames = samples.len() / channels.max(1);
                if frames == 0 {
                    return;
                }
                let peak_per_mille = peak_per_mille(&samples);
                let queued = QueuedAudioFrame {
                    frame: AudioFrame {
                        sample_rate,
                        channels,
                        samples,
                    },
                    frames,
                };
                match tx.try_send(queued) {
                    Ok(()) => {
                        stats
                            .captured_frames
                            .fetch_add(frames as u64, Ordering::Relaxed);
                        stats
                            .queued_frames
                            .fetch_add(frames as u64, Ordering::Relaxed);
                        stats
                            .input_peak_per_mille
                            .store(peak_per_mille, Ordering::Relaxed);
                    }
                    Err(TrySendError::Full(_)) => {
                        stats.input_drops.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(TrySendError::Disconnected(_)) => {}
                }
            },
            |error| eprintln!("Streaming input CPAL error: {error}"),
            None,
        )
        .map_err(|error| format!("Input stream error: {error}"))
}

fn frames_to_ms(frames: u64, sample_rate: u64) -> u64 {
    frames.saturating_mul(1000) / sample_rate.max(1)
}

fn peak_per_mille(samples: &[f32]) -> u64 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0)
        .mul_add(1000.0, 0.0)
        .round() as u64
}
