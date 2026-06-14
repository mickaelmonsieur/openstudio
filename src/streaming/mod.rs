use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crate::db;
use rubato::{
    audioadapter_buffers::direct::InterleavedSlice, calculate_cutoff, Async, FixedAsync, Resampler,
    SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

mod encoder;
mod fdk_aac;
mod input;
#[cfg(openstudio_has_lame)]
mod lame;
#[cfg(openstudio_has_shout)]
mod shout;

pub use input::InputDiagnostics;

const STREAM_INPUT_PREBUFFER_MS: u64 = 8_000;
const STREAM_INPUT_TIMEOUT_MS: u64 = 250;
const STREAM_ENCODED_QUEUE_CAPACITY: usize = 2048;
const INPUT_SILENCE_RESTART_MS: u64 = 3_000;
const SILENCE_PEAK_PER_MILLE: u64 = 1;

pub fn fdk_aac_available() -> bool {
    fdk_aac::is_available()
}

pub struct StreamingHandle {
    stop_tx: mpsc::Sender<()>,
    state: Arc<StreamingState>,
    metadata_title: Arc<Mutex<Option<String>>>,
}

impl StreamingHandle {
    pub fn stop(&self) {
        let _ = self.stop_tx.send(());
    }

    pub fn status(&self) -> String {
        self.state.status()
    }

    pub fn kbps(&self) -> u64 {
        self.state.kbps()
    }

    pub fn timing(&self) -> StreamingTiming {
        self.state.timing()
    }

    pub fn input_diagnostics(&self) -> InputDiagnostics {
        self.state.input_diagnostics()
    }

    pub fn set_title_metadata(&self, title: impl Into<String>) {
        let title = title.into();
        if let Ok(mut metadata_title) = self.metadata_title.lock() {
            *metadata_title = Some(title);
        }
    }
}

impl Drop for StreamingHandle {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StreamingConfig {
    pub bitrate_kbps: i32,
    pub sample_rate: i32,
    pub channels: i32,
    pub encoder_type: String,
    pub input_device: String,
    pub host: String,
    pub port: i32,
    pub password: String,
    pub mountpoint: String,
    pub reconnect_seconds: i32,
}

#[derive(Debug, Clone)]
pub struct StreamingTiming {
    pub launched_at: String,
    pub uptime: String,
    pub last_reconnect_at: String,
}

struct StreamingState {
    status: Mutex<String>,
    launched_at: SystemTime,
    launched_instant: Instant,
    last_connected_at: Mutex<Option<SystemTime>>,
    connected: AtomicU64,
    frames_encoded: AtomicU64,
    bytes_sent: AtomicU64,
    last_frame_at: Mutex<Option<Instant>>,
    last_send_at: Mutex<Option<Instant>>,
    bitrate_meter: Mutex<BitrateMeter>,
    input_diagnostics: Mutex<InputDiagnostics>,
    processed_peak_per_mille: AtomicU64,
    resampler_pending_ms: AtomicU64,
    encoded_queue_frames: AtomicU64,
    encoded_sample_rate: AtomicU64,
    input_silence_ms: AtomicU64,
    input_restarts: AtomicU64,
}

struct BitrateMeter {
    last_bytes: u64,
    last_at: Instant,
    kbps: u64,
}

impl StreamingState {
    fn new() -> Self {
        Self {
            status: Mutex::new(String::from("Starting")),
            launched_at: SystemTime::now(),
            launched_instant: Instant::now(),
            last_connected_at: Mutex::new(None),
            connected: AtomicU64::new(0),
            frames_encoded: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            last_frame_at: Mutex::new(None),
            last_send_at: Mutex::new(None),
            bitrate_meter: Mutex::new(BitrateMeter {
                last_bytes: 0,
                last_at: Instant::now(),
                kbps: 0,
            }),
            input_diagnostics: Mutex::new(InputDiagnostics::default()),
            processed_peak_per_mille: AtomicU64::new(0),
            resampler_pending_ms: AtomicU64::new(0),
            encoded_queue_frames: AtomicU64::new(0),
            encoded_sample_rate: AtomicU64::new(44_100),
            input_silence_ms: AtomicU64::new(0),
            input_restarts: AtomicU64::new(0),
        }
    }

    fn set_status(&self, value: impl Into<String>) {
        if let Ok(mut status) = self.status.lock() {
            *status = value.into();
        }
    }

    fn set_connected(&self, connected: bool) {
        self.connected
            .store(u64::from(connected), Ordering::Relaxed);
        if connected {
            if let Ok(mut last_connected_at) = self.last_connected_at.lock() {
                *last_connected_at = Some(SystemTime::now());
            }
        }
    }

    fn note_frame(&self, frames: usize) {
        self.frames_encoded
            .fetch_add(frames as u64, Ordering::Relaxed);
        if let Ok(mut last_frame_at) = self.last_frame_at.lock() {
            *last_frame_at = Some(Instant::now());
        }
    }

    fn note_encoded_queued(&self, frames: usize, sample_rate: u32) {
        self.encoded_queue_frames
            .fetch_add(frames as u64, Ordering::Relaxed);
        self.encoded_sample_rate
            .store(sample_rate as u64, Ordering::Relaxed);
    }

    fn note_encoded_dequeued(&self, frames: usize) {
        self.encoded_queue_frames
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(frames as u64))
            })
            .ok();
    }

    fn set_processing_diagnostics(&self, processed_peak_per_mille: u64, resampler_pending_ms: u64) {
        self.processed_peak_per_mille
            .store(processed_peak_per_mille, Ordering::Relaxed);
        self.resampler_pending_ms
            .store(resampler_pending_ms, Ordering::Relaxed);
    }

    fn set_input_silence_ms(&self, silence_ms: u64) {
        self.input_silence_ms.store(silence_ms, Ordering::Relaxed);
    }

    fn note_input_restart(&self) {
        self.input_restarts.fetch_add(1, Ordering::Relaxed);
        self.input_silence_ms.store(0, Ordering::Relaxed);
    }

    fn note_send(&self, bytes: usize) {
        self.bytes_sent.fetch_add(bytes as u64, Ordering::Relaxed);
        if let Ok(mut last_send_at) = self.last_send_at.lock() {
            *last_send_at = Some(Instant::now());
        }
    }

    fn status(&self) -> String {
        let base = self
            .status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_else(|_| String::from("Status unavailable"));
        if self.connected.load(Ordering::Relaxed) == 0 {
            return base;
        }

        let last_send_elapsed = self
            .last_send_at
            .lock()
            .ok()
            .and_then(|last_send_at| last_send_at.as_ref().map(Instant::elapsed));
        let last_frame_elapsed = self
            .last_frame_at
            .lock()
            .ok()
            .and_then(|last_frame_at| last_frame_at.as_ref().map(Instant::elapsed));

        match (last_send_elapsed, last_frame_elapsed) {
            (Some(send_elapsed), _) if send_elapsed <= Duration::from_secs(3) => {
                format!(
                    "Connected - streaming ({} KB)",
                    self.bytes_sent.load(Ordering::Relaxed) / 1024
                )
            }
            (_, Some(frame_elapsed)) if frame_elapsed <= Duration::from_secs(3) => {
                String::from("Connected - encoding, no send")
            }
            _ => String::from("Connected - no audio"),
        }
    }

    fn kbps(&self) -> u64 {
        let total_bytes = self.bytes_sent.load(Ordering::Relaxed);
        let Ok(mut meter) = self.bitrate_meter.lock() else {
            return 0;
        };
        let elapsed = meter.last_at.elapsed();
        if elapsed >= Duration::from_millis(500) {
            let delta_bytes = total_bytes.saturating_sub(meter.last_bytes);
            let bits_per_second = delta_bytes as f64 * 8.0 / elapsed.as_secs_f64().max(0.001);
            meter.kbps = (bits_per_second / 1000.0).round().max(0.0) as u64;
            meter.last_bytes = total_bytes;
            meter.last_at = Instant::now();
        }
        meter.kbps
    }

    fn timing(&self) -> StreamingTiming {
        let last_reconnect_at = self
            .last_connected_at
            .lock()
            .ok()
            .and_then(|last_connected_at| *last_connected_at)
            .map(format_system_time)
            .unwrap_or_else(|| String::from("-"));
        StreamingTiming {
            launched_at: format_system_time(self.launched_at),
            uptime: format_duration(self.launched_instant.elapsed()),
            last_reconnect_at,
        }
    }

    fn set_input_diagnostics(&self, diagnostics: InputDiagnostics) {
        if let Ok(mut current) = self.input_diagnostics.lock() {
            *current = diagnostics;
        }
    }

    fn input_diagnostics(&self) -> InputDiagnostics {
        let mut diagnostics = self
            .input_diagnostics
            .lock()
            .map(|diagnostics| *diagnostics)
            .unwrap_or_default();
        diagnostics.processed_peak_per_mille =
            self.processed_peak_per_mille.load(Ordering::Relaxed);
        diagnostics.resampler_pending_ms = self.resampler_pending_ms.load(Ordering::Relaxed);
        diagnostics.encoded_queue_ms = frames_to_ms(
            self.encoded_queue_frames.load(Ordering::Relaxed),
            self.encoded_sample_rate.load(Ordering::Relaxed),
        );
        diagnostics.input_silence_ms = self.input_silence_ms.load(Ordering::Relaxed);
        diagnostics.input_restarts = self.input_restarts.load(Ordering::Relaxed);
        diagnostics
    }
}

impl From<&db::AppConfig> for StreamingConfig {
    fn from(cfg: &db::AppConfig) -> Self {
        Self {
            bitrate_kbps: cfg.encoder_bitrate.clamp(8, 320),
            sample_rate: cfg.encoder_sample_rate.clamp(8_000, 48_000),
            channels: cfg.encoder_channels.clamp(1, 2),
            encoder_type: cfg.encoder_type.clone(),
            input_device: cfg.encoder_input_device_id.clone(),
            host: cfg.encoder_server_host.clone(),
            port: cfg.encoder_server_port.clamp(1, 65_535),
            password: cfg.encoder_password.clone(),
            mountpoint: normalized_mountpoint(&cfg.encoder_mountpoint),
            reconnect_seconds: cfg.encoder_reconnect_seconds.clamp(1, 3_600),
        }
    }
}

pub fn start(config: StreamingConfig) -> StreamingHandle {
    let (stop_tx, stop_rx) = mpsc::channel();
    let config = Arc::new(Mutex::new(config));
    let state = Arc::new(StreamingState::new());
    let metadata_title = Arc::new(Mutex::new(None));
    let worker_config = Arc::clone(&config);
    let worker_state = Arc::clone(&state);
    let worker_metadata_title = Arc::clone(&metadata_title);
    thread::spawn(move || run(worker_config, worker_state, worker_metadata_title, stop_rx));
    StreamingHandle {
        stop_tx,
        state,
        metadata_title,
    }
}

fn run(
    config: Arc<Mutex<StreamingConfig>>,
    state: Arc<StreamingState>,
    metadata_title: Arc<Mutex<Option<String>>>,
    stop_rx: mpsc::Receiver<()>,
) {
    #[cfg(all(openstudio_has_lame, openstudio_has_shout))]
    {
        run_lame_shout(config, state, metadata_title, stop_rx);
    }

    #[cfg(not(all(openstudio_has_lame, openstudio_has_shout)))]
    {
        run_noop(config, state, metadata_title, stop_rx);
    }
}

#[cfg(all(openstudio_has_lame, openstudio_has_shout))]
fn run_lame_shout(
    config: Arc<Mutex<StreamingConfig>>,
    state: Arc<StreamingState>,
    metadata_title: Arc<Mutex<Option<String>>>,
    stop_rx: mpsc::Receiver<()>,
) {
    loop {
        if stop_rx.try_recv().is_ok() {
            return;
        }

        let current_config = config_snapshot(&config);
        state.set_connected(false);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let (encoded_tx, encoded_rx) = mpsc::sync_channel(STREAM_ENCODED_QUEUE_CAPACITY);
        let (error_tx, error_rx) = mpsc::channel::<String>();
        let (ready_tx, ready_rx) = mpsc::channel::<EncoderReady>();

        let encoder_thread = {
            let config = current_config.clone();
            let state = Arc::clone(&state);
            let stop_flag = Arc::clone(&stop_flag);
            let error_tx = error_tx.clone();
            thread::spawn(move || {
                run_encoder_worker(config, state, encoded_tx, stop_flag, error_tx, ready_tx)
            })
        };

        let mut network_thread = None;
        let mut encoded_rx = Some(encoded_rx);

        let mut should_reconnect = true;
        loop {
            if stop_rx.try_recv().is_ok() {
                stop_flag.store(true, Ordering::Relaxed);
                should_reconnect = false;
                break;
            }
            if network_thread.is_none() {
                if let Ok(ready) = ready_rx.try_recv() {
                    if let Some(encoded_rx) = encoded_rx.take() {
                        let config = current_config.clone();
                        let state = Arc::clone(&state);
                        let stop_flag = Arc::clone(&stop_flag);
                        let metadata_title = Arc::clone(&metadata_title);
                        let error_tx = error_tx.clone();
                        network_thread = Some(thread::spawn(move || {
                            run_network_worker(
                                config,
                                encoded_rx,
                                state,
                                metadata_title,
                                stop_flag,
                                error_tx,
                                ready.format,
                            )
                        }));
                    }
                }
            }
            match error_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(error) => {
                    stop_flag.store(true, Ordering::Relaxed);
                    state.set_connected(false);
                    state.set_status(error);
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    stop_flag.store(true, Ordering::Relaxed);
                    state.set_connected(false);
                    state.set_status(String::from("Streaming workers stopped"));
                    break;
                }
            }
        }

        let _ = encoder_thread.join();
        if let Some(network_thread) = network_thread {
            let _ = network_thread.join();
        }
        if !should_reconnect {
            return;
        }

        sleep_reconnect(&config_snapshot(&config), &stop_rx);
    }
}

struct EncodedPacket {
    bytes: Vec<u8>,
    frames: usize,
}

struct EncoderReady {
    format: encoder::EncodedAudioFormat,
}

fn run_encoder_worker(
    config: StreamingConfig,
    state: Arc<StreamingState>,
    encoded_tx: SyncSender<EncodedPacket>,
    stop_flag: Arc<AtomicBool>,
    error_tx: mpsc::Sender<String>,
    ready_tx: mpsc::Sender<EncoderReady>,
) {
    let mut input = match input::StreamInput::open(input_device_name(&config).as_deref()) {
        Ok(input) => input,
        Err(error) => {
            let _ = error_tx.send(format!("Input error: {error}"));
            return;
        }
    };
    if !prebuffer_input(&input, &state, &stop_flag) {
        return;
    }
    let mut encoder = match encoder::new_encoder(&config) {
        Ok(encoder) => encoder,
        Err(error) => {
            let _ = error_tx.send(format!("Encoder error: {error}"));
            return;
        }
    };
    let format = encoder.format();
    let frame_samples = encoder.frame_samples();
    let _ = ready_tx.send(EncoderReady { format });
    let mut converter = PcmConverter::new(config.channels.clamp(1, 2) as usize, config.sample_rate);
    let mut chunker = PcmChunker::new(
        config.channels.clamp(1, 2) as usize,
        config.sample_rate,
        frame_samples,
    );
    let mut saw_input_audio = false;
    let mut input_silence_ms = 0_u64;
    while !stop_flag.load(Ordering::Relaxed) {
        state.set_input_diagnostics(input.diagnostics());
        let raw_frame = match input.recv_timeout(Duration::from_millis(STREAM_INPUT_TIMEOUT_MS)) {
            Ok(frame) => frame,
            Err(RecvTimeoutError::Timeout) => {
                input.note_timeout();
                state.set_input_diagnostics(input.diagnostics());
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => {
                let _ = error_tx.send(String::from("Input capture disconnected"));
                return;
            }
        };
        let raw_peak = peak_per_mille(&raw_frame.samples);
        if raw_peak > SILENCE_PEAK_PER_MILLE {
            saw_input_audio = true;
            input_silence_ms = 0;
            state.set_input_silence_ms(0);
        } else if saw_input_audio {
            input_silence_ms = input_silence_ms.saturating_add(frame_ms(
                raw_frame.samples.len() / raw_frame.channels.max(1),
                raw_frame.sample_rate,
            ));
            state.set_input_silence_ms(input_silence_ms);
            if input_silence_ms >= INPUT_SILENCE_RESTART_MS {
                state.set_status(String::from("Restarting silent input"));
                match input::StreamInput::open(input_device_name(&config).as_deref()) {
                    Ok(new_input) => {
                        input = new_input;
                        converter = PcmConverter::new(
                            config.channels.clamp(1, 2) as usize,
                            config.sample_rate,
                        );
                        chunker = PcmChunker::new(
                            config.channels.clamp(1, 2) as usize,
                            config.sample_rate,
                            frame_samples,
                        );
                        state.note_input_restart();
                        input_silence_ms = 0;
                        saw_input_audio = true;
                        continue;
                    }
                    Err(error) => {
                        let _ = error_tx.send(format!("Input restart error: {error}"));
                        return;
                    }
                }
            }
        }

        let frame = match converter.convert(raw_frame) {
            Ok(frame) => frame,
            Err(error) => {
                let _ = error_tx.send(format!("Resample error: {error}"));
                return;
            }
        };
        state.set_input_diagnostics(input.diagnostics());
        state.set_processing_diagnostics(peak_per_mille(&frame.samples), converter.pending_ms());
        if frame.samples.is_empty() {
            continue;
        }
        for frame in chunker.push(frame) {
            let frames = frame.samples.len() / frame.channels.max(1);
            let encoded = match encoder.encode(&frame) {
                Ok(encoded) => encoded,
                Err(error) => {
                    let _ = error_tx.send(format!("Encode error: {error}"));
                    return;
                }
            };
            if encoded.is_empty() {
                continue;
            }
            if encoded_tx
                .send(EncodedPacket {
                    bytes: encoded,
                    frames,
                })
                .is_err()
            {
                return;
            }
            state.note_frame(frames);
            state.note_encoded_queued(frames, frame.sample_rate);
        }
    }
}

fn run_network_worker(
    config: StreamingConfig,
    encoded_rx: mpsc::Receiver<EncodedPacket>,
    state: Arc<StreamingState>,
    metadata_title: Arc<Mutex<Option<String>>>,
    stop_flag: Arc<AtomicBool>,
    error_tx: mpsc::Sender<String>,
    format: encoder::EncodedAudioFormat,
) {
    state.set_status(format!("Connecting to {}", config.mountpoint));
    let mut client = match shout::IcecastClient::connect(&config, format) {
        Ok(client) => client,
        Err(error) => {
            let _ = error_tx.send(format!("Icecast connection error: {error}"));
            return;
        }
    };
    state.set_connected(true);
    state.set_status(String::from("Connected"));
    let mut sent_metadata_title = send_pending_metadata(&mut client, &metadata_title, None);

    while !stop_flag.load(Ordering::Relaxed) {
        sent_metadata_title =
            send_pending_metadata(&mut client, &metadata_title, sent_metadata_title);
        let packet = match encoded_rx.recv_timeout(Duration::from_millis(250)) {
            Ok(packet) => packet,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        };
        if let Err(error) = client.send(&packet.bytes) {
            let _ = error_tx.send(format!("Icecast send error: {error}"));
            return;
        }
        state.note_send(packet.bytes.len());
        state.note_encoded_dequeued(packet.frames);
    }
}

struct PcmChunker {
    channels: usize,
    sample_rate: u32,
    frame_samples: usize,
    samples: Vec<f32>,
}

impl PcmChunker {
    fn new(channels: usize, sample_rate: i32, frame_samples: usize) -> Self {
        let channels = channels.max(1);
        let frame_samples = frame_samples.max(1);
        Self {
            channels,
            sample_rate: sample_rate.clamp(8_000, 48_000) as u32,
            frame_samples,
            samples: Vec::with_capacity(frame_samples * channels * 4),
        }
    }

    fn push(&mut self, frame: input::AudioFrame) -> Vec<input::AudioFrame> {
        self.samples.extend(frame.samples);
        let chunk_samples = self.frame_samples * self.channels;
        let mut chunks = Vec::new();
        while self.samples.len() >= chunk_samples {
            let samples = self.samples.drain(..chunk_samples).collect::<Vec<_>>();
            chunks.push(input::AudioFrame {
                sample_rate: self.sample_rate,
                channels: self.channels,
                samples,
            });
        }
        chunks
    }
}

struct PcmConverter {
    target_channels: usize,
    target_sample_rate: u32,
    resampler: Option<RubatoResampler>,
}

impl PcmConverter {
    fn new(target_channels: usize, target_sample_rate: i32) -> Self {
        Self {
            target_channels: target_channels.clamp(1, 2),
            target_sample_rate: target_sample_rate.clamp(8_000, 48_000) as u32,
            resampler: None,
        }
    }

    fn convert(&mut self, frame: input::AudioFrame) -> Result<input::AudioFrame, String> {
        let samples = convert_channels(&frame.samples, frame.channels, self.target_channels);
        if frame.sample_rate == self.target_sample_rate {
            self.resampler = None;
            return Ok(input::AudioFrame {
                sample_rate: self.target_sample_rate,
                channels: self.target_channels,
                samples,
            });
        }

        let resampler = self.resampler.get_or_insert_with(|| {
            RubatoResampler::new(
                frame.sample_rate,
                self.target_sample_rate,
                self.target_channels,
            )
        });
        if !resampler.matches(
            frame.sample_rate,
            self.target_sample_rate,
            self.target_channels,
        ) {
            *resampler = RubatoResampler::new(
                frame.sample_rate,
                self.target_sample_rate,
                self.target_channels,
            );
        }

        Ok(input::AudioFrame {
            sample_rate: self.target_sample_rate,
            channels: self.target_channels,
            samples: resampler.push(&samples)?,
        })
    }

    fn pending_ms(&self) -> u64 {
        self.resampler
            .as_ref()
            .map(RubatoResampler::pending_ms)
            .unwrap_or_default()
    }
}

struct RubatoResampler {
    from_rate: u32,
    to_rate: u32,
    channels: usize,
    input: Vec<f32>,
    resampler: Box<dyn Resampler<f32>>,
}

impl RubatoResampler {
    fn new(from_rate: u32, to_rate: u32, channels: usize) -> Self {
        let channels = channels.max(1);
        let sinc_len = 128;
        let window = WindowFunction::Blackman2;
        let params = SincInterpolationParameters {
            sinc_len,
            f_cutoff: calculate_cutoff(sinc_len, window),
            interpolation: SincInterpolationType::Quadratic,
            oversampling_factor: 256,
            window,
        };
        let ratio = to_rate as f64 / from_rate.max(1) as f64;
        let resampler =
            Async::<f32>::new_sinc(ratio, 1.1, &params, 1024, channels, FixedAsync::Input)
                .expect("valid rubato streaming resampler configuration");
        Self {
            from_rate,
            to_rate,
            channels,
            input: Vec::with_capacity(4096 * channels),
            resampler: Box::new(resampler),
        }
    }

    fn matches(&self, from_rate: u32, to_rate: u32, channels: usize) -> bool {
        self.from_rate == from_rate && self.to_rate == to_rate && self.channels == channels.max(1)
    }

    fn push(&mut self, samples: &[f32]) -> Result<Vec<f32>, String> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        self.input.extend_from_slice(samples);
        let mut output = Vec::new();
        let needed_frames = self.resampler.input_frames_next();
        let needed_samples = needed_frames * self.channels;
        while self.input.len() >= needed_samples {
            let input_adapter =
                InterleavedSlice::new(&self.input[..needed_samples], self.channels, needed_frames)
                    .map_err(|error| error.to_string())?;
            let resampled = self
                .resampler
                .process(&input_adapter, 0, None)
                .map_err(|error| error.to_string())?;
            output.extend(resampled.take_data());
            self.input.drain(..needed_samples);
        }

        Ok(output)
    }

    fn pending_ms(&self) -> u64 {
        let frames = self.input.len() / self.channels.max(1);
        frames_to_ms(frames as u64, self.from_rate as u64)
    }
}

fn prebuffer_input(
    input: &input::StreamInput,
    state: &StreamingState,
    stop_flag: &AtomicBool,
) -> bool {
    state.set_status(format!(
        "Buffering input ({} ms)",
        STREAM_INPUT_PREBUFFER_MS
    ));
    loop {
        if stop_flag.load(Ordering::Relaxed) {
            return false;
        }
        let diagnostics = input.diagnostics();
        state.set_input_diagnostics(diagnostics);
        if diagnostics.queue_ms >= STREAM_INPUT_PREBUFFER_MS {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(not(all(openstudio_has_lame, openstudio_has_shout)))]
fn run_noop(
    config: Arc<Mutex<StreamingConfig>>,
    state: Arc<StreamingState>,
    _metadata_title: Arc<Mutex<Option<String>>>,
    stop_rx: mpsc::Receiver<()>,
) {
    let config = config_snapshot(&config);
    state.set_status(format!(
        "Unavailable: libmp3lame/libshout not found at build time ({}:{}{})",
        config.host, config.port, config.mountpoint
    ));
    loop {
        if stop_rx.try_recv().is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(250));
    }
}

#[cfg(all(openstudio_has_lame, openstudio_has_shout))]
fn send_pending_metadata(
    client: &mut shout::IcecastClient,
    metadata_title: &Arc<Mutex<Option<String>>>,
    last_sent: Option<String>,
) -> Option<String> {
    let pending = metadata_title
        .lock()
        .ok()
        .and_then(|metadata_title| metadata_title.clone());
    let Some(title) = pending else {
        return last_sent;
    };
    if last_sent.as_deref() == Some(title.as_str()) {
        return last_sent;
    }
    match client.set_song_title(&title) {
        Ok(()) => Some(title),
        Err(error) => {
            if !error.eq_ignore_ascii_case("no error") {
                eprintln!("Icecast metadata error: {error}");
            }
            last_sent
        }
    }
}

fn config_snapshot(config: &Arc<Mutex<StreamingConfig>>) -> StreamingConfig {
    config
        .lock()
        .map(|config| config.clone())
        .unwrap_or_else(|_| StreamingConfig {
            bitrate_kbps: 128,
            sample_rate: 44_100,
            channels: 2,
            encoder_type: String::from("mp3"),
            input_device: String::new(),
            host: String::from("openstudio.entrypoint.belstream.net"),
            port: 80,
            password: String::new(),
            mountpoint: String::from("/live"),
            reconnect_seconds: 10,
        })
}

fn convert_channels(samples: &[f32], from_channels: usize, to_channels: usize) -> Vec<f32> {
    match (from_channels, to_channels) {
        (2, 2) => samples.to_vec(),
        (2, 1) => samples
            .chunks_exact(2)
            .map(|frame| (frame[0] + frame[1]) * 0.5)
            .collect(),
        (1, 2) => samples
            .iter()
            .flat_map(|&sample| [sample, sample])
            .collect(),
        (1, 1) => samples.to_vec(),
        (_, 1) => samples
            .chunks(from_channels.max(1))
            .map(|frame| frame.iter().copied().sum::<f32>() / frame.len().max(1) as f32)
            .collect(),
        (_, 2) => samples
            .chunks(from_channels.max(1))
            .flat_map(|frame| {
                let left = frame.first().copied().unwrap_or_default();
                let right = frame.get(1).copied().unwrap_or(left);
                [left, right]
            })
            .collect(),
        _ => Vec::new(),
    }
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

fn frames_to_ms(frames: u64, sample_rate: u64) -> u64 {
    frames.saturating_mul(1000) / sample_rate.max(1)
}

fn frame_ms(frames: usize, sample_rate: u32) -> u64 {
    frames_to_ms(frames as u64, sample_rate as u64)
}

fn input_device_name(config: &StreamingConfig) -> Option<String> {
    let trimmed = config.input_device.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let days = total / 86_400;
    let hours = (total / 3_600) % 24;
    let minutes = (total / 60) % 60;
    let seconds = total % 60;
    if days > 0 {
        format!("{days}d {hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}

#[cfg(unix)]
fn format_system_time(time: SystemTime) -> String {
    let Ok(since_epoch) = time.duration_since(std::time::UNIX_EPOCH) else {
        return String::from("-");
    };
    let timestamp = since_epoch.as_secs() as libc::time_t;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
    let local = unsafe {
        if libc::localtime_r(&timestamp, local.as_mut_ptr()).is_null() {
            return String::from("-");
        }
        local.assume_init()
    };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        local.tm_year + 1900,
        local.tm_mon + 1,
        local.tm_mday,
        local.tm_hour,
        local.tm_min,
        local.tm_sec
    )
}

#[cfg(not(unix))]
fn format_system_time(time: SystemTime) -> String {
    let Ok(since_epoch) = time.duration_since(std::time::UNIX_EPOCH) else {
        return String::from("-");
    };
    format!("{}s since epoch", since_epoch.as_secs())
}

#[cfg(all(openstudio_has_lame, openstudio_has_shout))]
fn sleep_reconnect(config: &StreamingConfig, stop_rx: &mpsc::Receiver<()>) {
    let sleep = Duration::from_secs(config.reconnect_seconds.max(1) as u64);
    let started = std::time::Instant::now();
    while started.elapsed() < sleep {
        if stop_rx.try_recv().is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn normalized_mountpoint(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        String::from("/live")
    } else if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}
