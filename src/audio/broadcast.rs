use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use super::PlayerId;

pub const BROADCAST_SAMPLE_RATE: u32 = 44_100;
pub const BROADCAST_CHANNELS: usize = 2;

const INPUT_QUEUE_CAPACITY: usize = 256;
const OUTPUT_QUEUE_CAPACITY: usize = 64;
const MIX_CHUNK_FRAMES: usize = 1024;
const MIX_IDLE_SLEEP: Duration = Duration::from_millis(2);
const MIX_FLUSH_AFTER: Duration = Duration::from_millis(20);
const CLIP_WARNING_THRESHOLD: f32 = 0.98;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BroadcastFrame {
    pub sample_rate: u32,
    pub channels: usize,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BroadcastDiagnostics {
    pub input_drops: u64,
    pub output_drops: u64,
    pub source_underruns: u64,
    pub clip_warnings: u64,
    pub mixed_frames: u64,
}

#[derive(Default)]
struct BroadcastStats {
    input_drops: AtomicU64,
    output_drops: AtomicU64,
    source_underruns: AtomicU64,
    clip_warnings: AtomicU64,
    mixed_frames: AtomicU64,
}

impl BroadcastStats {
    fn snapshot(&self) -> BroadcastDiagnostics {
        BroadcastDiagnostics {
            input_drops: self.input_drops.load(Ordering::Relaxed),
            output_drops: self.output_drops.load(Ordering::Relaxed),
            source_underruns: self.source_underruns.load(Ordering::Relaxed),
            clip_warnings: self.clip_warnings.load(Ordering::Relaxed),
            mixed_frames: self.mixed_frames.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone)]
pub struct BroadcastBus {
    input_tx: SyncSender<BroadcastInput>,
    output_rx: std::sync::Arc<Mutex<Option<Receiver<BroadcastFrame>>>>,
    stats: Arc<BroadcastStats>,
}

impl BroadcastBus {
    pub fn new() -> Self {
        let (input_tx, input_rx) = mpsc::sync_channel(INPUT_QUEUE_CAPACITY);
        let (output_tx, output_rx) = mpsc::sync_channel(OUTPUT_QUEUE_CAPACITY);
        let stats = Arc::new(BroadcastStats::default());
        let worker_stats = Arc::clone(&stats);
        thread::spawn(move || run_mixer(input_rx, output_tx, worker_stats));
        Self {
            input_tx,
            output_rx: std::sync::Arc::new(Mutex::new(Some(output_rx))),
            stats,
        }
    }

    pub fn push_player_buffer(
        &self,
        player_id: PlayerId,
        samples: &[f32],
        channels: usize,
        sample_rate: u32,
    ) {
        if samples.is_empty() || channels == 0 || sample_rate == 0 {
            return;
        }

        let input = BroadcastInput {
            player_id,
            sample_rate,
            channels,
            samples: samples.to_vec(),
        };

        match self.input_tx.try_send(input) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                self.stats.input_drops.fetch_add(1, Ordering::Relaxed);
            }
            Err(TrySendError::Disconnected(_)) => {}
        }
    }

    #[allow(dead_code)]
    pub fn take_output(&self) -> Option<Receiver<BroadcastFrame>> {
        self.output_rx.lock().ok()?.take()
    }

    pub fn diagnostics(&self) -> BroadcastDiagnostics {
        self.stats.snapshot()
    }
}

impl Default for BroadcastBus {
    fn default() -> Self {
        Self::new()
    }
}

struct BroadcastInput {
    player_id: PlayerId,
    sample_rate: u32,
    channels: usize,
    samples: Vec<f32>,
}

fn run_mixer(
    input_rx: Receiver<BroadcastInput>,
    output_tx: SyncSender<BroadcastFrame>,
    stats: Arc<BroadcastStats>,
) {
    let mut sources: HashMap<PlayerId, VecDeque<f32>> = HashMap::new();
    let mut last_input_at = Instant::now();

    loop {
        while let Ok(input) = input_rx.try_recv() {
            let stereo = to_broadcast_stereo(&input.samples, input.channels, input.sample_rate);
            if !stereo.is_empty() {
                sources.entry(input.player_id).or_default().extend(stereo);
                last_input_at = Instant::now();
            }
        }

        sources.retain(|_, queue| !queue.is_empty());

        let max_frames = sources
            .values()
            .map(|queue| queue.len() / BROADCAST_CHANNELS)
            .max()
            .unwrap_or(0);

        let should_flush_partial = max_frames > 0 && last_input_at.elapsed() >= MIX_FLUSH_AFTER;
        if max_frames >= MIX_CHUNK_FRAMES || should_flush_partial {
            let frames = max_frames.min(MIX_CHUNK_FRAMES);
            let samples = mix_next_frames(&mut sources, frames, &stats);
            let frame = BroadcastFrame {
                sample_rate: BROADCAST_SAMPLE_RATE,
                channels: BROADCAST_CHANNELS,
                samples,
            };
            match output_tx.try_send(frame) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => {
                    stats.output_drops.fetch_add(1, Ordering::Relaxed);
                }
                Err(TrySendError::Disconnected(_)) => break,
            }
        } else {
            match input_rx.recv_timeout(MIX_IDLE_SLEEP) {
                Ok(input) => {
                    let stereo =
                        to_broadcast_stereo(&input.samples, input.channels, input.sample_rate);
                    if !stereo.is_empty() {
                        sources.entry(input.player_id).or_default().extend(stereo);
                        last_input_at = Instant::now();
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }
}

fn mix_next_frames(
    sources: &mut HashMap<PlayerId, VecDeque<f32>>,
    frames: usize,
    stats: &BroadcastStats,
) -> Vec<f32> {
    let mut out = vec![0.0; frames * BROADCAST_CHANNELS];
    let mut active_sources = 0usize;

    for queue in sources.values_mut() {
        if queue.is_empty() {
            continue;
        }
        active_sources += 1;
        let expected_samples = frames * BROADCAST_CHANNELS;
        if queue.len() < expected_samples {
            stats.source_underruns.fetch_add(1, Ordering::Relaxed);
        }
        for sample in out.iter_mut() {
            if let Some(value) = queue.pop_front() {
                *sample += value;
            }
        }
    }

    let gain = if active_sources > 1 {
        1.0 / (active_sources as f32).sqrt()
    } else {
        1.0
    };
    for sample in &mut out {
        let mixed = *sample * gain;
        if mixed.abs() >= CLIP_WARNING_THRESHOLD {
            stats.clip_warnings.fetch_add(1, Ordering::Relaxed);
        }
        *sample = soft_limit(mixed);
    }
    stats
        .mixed_frames
        .fetch_add(frames as u64, Ordering::Relaxed);
    out
}

fn to_broadcast_stereo(samples: &[f32], channels: usize, sample_rate: u32) -> Vec<f32> {
    let stereo = to_stereo(samples, channels);
    if sample_rate == BROADCAST_SAMPLE_RATE {
        return stereo;
    }
    resample_stereo(&stereo, sample_rate, BROADCAST_SAMPLE_RATE)
}

fn to_stereo(samples: &[f32], channels: usize) -> Vec<f32> {
    match channels {
        0 => Vec::new(),
        1 => samples
            .iter()
            .flat_map(|&sample| [sample, sample])
            .collect(),
        2 => samples.to_vec(),
        n => {
            let frames = samples.len() / n;
            let mut out = Vec::with_capacity(frames * BROADCAST_CHANNELS);
            for frame in samples.chunks(n).take(frames) {
                out.push(frame[0]);
                out.push(frame[1]);
            }
            out
        }
    }
}

fn resample_stereo(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if samples.len() < BROADCAST_CHANNELS || from_rate == 0 || to_rate == 0 {
        return Vec::new();
    }

    let in_frames = samples.len() / BROADCAST_CHANNELS;
    let out_frames = ((in_frames as u64 * to_rate as u64) / from_rate as u64).max(1) as usize;
    let ratio = from_rate as f64 / to_rate as f64;
    let mut out = Vec::with_capacity(out_frames * BROADCAST_CHANNELS);

    for out_index in 0..out_frames {
        let src_pos = out_index as f64 * ratio;
        let base = src_pos.floor() as usize;
        let frac = (src_pos - base as f64) as f32;
        let next = (base + 1).min(in_frames.saturating_sub(1));
        for channel in 0..BROADCAST_CHANNELS {
            let a = samples[base * BROADCAST_CHANNELS + channel];
            let b = samples[next * BROADCAST_CHANNELS + channel];
            out.push(a + (b - a) * frac);
        }
    }

    out
}

fn soft_limit(sample: f32) -> f32 {
    sample.clamp(-4.0, 4.0).tanh()
}
