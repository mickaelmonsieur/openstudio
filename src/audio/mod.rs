use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender, SyncSender, TrySendError};
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, SupportedStreamConfig};
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerId {
    QueueA,
    QueueB,
    Instant,
    Aux1,
    Aux2,
    Aux3,
    Preview,
}

impl PlayerId {
    pub const ALL: [Self; 7] = [
        Self::QueueA,
        Self::QueueB,
        Self::Instant,
        Self::Aux1,
        Self::Aux2,
        Self::Aux3,
        Self::Preview,
    ];
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Load(PathBuf),
    Play,
    Pause,
    Resume,
    TogglePause,
    Stop,
    Restart,
    SeekRelative(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Empty,
    Loaded,
    Playing,
    Paused,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub state: PlayerState,
    pub loaded_path: Option<PathBuf>,
    pub position: Duration,
    pub duration: Option<Duration>,
}

pub struct AudioManager {
    players: HashMap<PlayerId, AudioPlayer>,
}

impl AudioManager {
    pub fn new() -> Self {
        let players = PlayerId::ALL
            .into_iter()
            .map(|id| (id, AudioPlayer::new(id)))
            .collect();

        Self { players }
    }

    pub fn player(&self, id: PlayerId) -> &AudioPlayer {
        self.players
            .get(&id)
            .expect("audio manager initialized with all players")
    }

    pub fn player_mut(&mut self, id: PlayerId) -> &mut AudioPlayer {
        self.players
            .get_mut(&id)
            .expect("audio manager initialized with all players")
    }

    pub fn handle(&mut self, id: PlayerId, command: PlayerCommand) {
        self.player_mut(id).handle(command);
    }

    pub fn poll(&mut self) {
        for player in self.players.values_mut() {
            player.poll();
        }
    }

    pub fn any_active(&self) -> bool {
        self.players.values().any(AudioPlayer::is_active)
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AudioPlayer {
    id: PlayerId,
    loaded_path: Option<PathBuf>,
    duration: Option<Duration>,
    levels: Arc<AudioLevels>,
    stop_tx: Option<Sender<()>>,
    seek_tx: Option<Sender<SeekRequest>>,
    pause_tx: Option<Sender<bool>>,
    done_rx: Option<mpsc::Receiver<()>>,
    preload_rx: Option<mpsc::Receiver<Option<Preloaded>>>,
    position_ms: Option<Arc<AtomicU64>>,
    paused: bool,
}

impl AudioPlayer {
    fn new(id: PlayerId) -> Self {
        Self {
            id,
            loaded_path: None,
            duration: None,
            levels: Arc::new(AudioLevels::default()),
            stop_tx: None,
            seek_tx: None,
            pause_tx: None,
            done_rx: None,
            preload_rx: None,
            position_ms: None,
            paused: false,
        }
    }

    pub fn handle(&mut self, command: PlayerCommand) {
        match command {
            PlayerCommand::Load(path) => self.load(path),
            PlayerCommand::Play => self.play(),
            PlayerCommand::Pause => self.pause(),
            PlayerCommand::Resume => self.resume(),
            PlayerCommand::TogglePause => self.toggle_pause(),
            PlayerCommand::Stop => self.stop(),
            PlayerCommand::Restart => self.restart(),
            PlayerCommand::SeekRelative(offset_ms) => self.seek_relative(offset_ms),
        }
    }

    pub fn load(&mut self, path: PathBuf) {
        self.stop_thread();
        self.duration = read_duration(&path);
        self.preload_rx = Some(preload(path.clone()));
        self.loaded_path = Some(path);
    }

    pub fn play(&mut self) {
        let Some(path) = self.loaded_path.clone() else {
            return;
        };

        self.stop_thread();
        let preloaded = self
            .preload_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok().flatten());
        self.preload_rx = None;

        let (stop_tx, seek_tx, pause_tx, position_ms, done_rx) =
            play(path, preloaded, Arc::clone(&self.levels));
        self.stop_tx = Some(stop_tx);
        self.seek_tx = Some(seek_tx);
        self.pause_tx = Some(pause_tx);
        self.position_ms = Some(position_ms);
        self.done_rx = Some(done_rx);
        self.paused = false;
    }

    pub fn pause(&mut self) {
        if let Some(tx) = &self.pause_tx {
            let _ = tx.send(true);
            self.paused = true;
            self.levels.reset();
        }
    }

    pub fn resume(&mut self) {
        if let Some(tx) = &self.pause_tx {
            let _ = tx.send(false);
            self.paused = false;
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.stop_tx.is_some() && self.paused {
            self.resume();
        } else if self.stop_tx.is_some() {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn stop(&mut self) {
        self.stop_thread();
        self.preload_rx = self.loaded_path.as_ref().map(|path| preload(path.clone()));
    }

    pub fn seek_relative(&mut self, offset_ms: i64) {
        if let Some(tx) = &self.seek_tx {
            let _ = tx.send(SeekRequest::Relative(offset_ms));
        }
    }

    pub fn restart(&mut self) {
        if let Some(tx) = &self.seek_tx {
            let _ = tx.send(SeekRequest::Absolute(Duration::ZERO));
        }
    }

    pub fn poll(&mut self) {
        use std::sync::mpsc::TryRecvError;

        let finished = self
            .done_rx
            .as_ref()
            .is_some_and(|rx| !matches!(rx.try_recv(), Err(TryRecvError::Empty)));

        if finished {
            self.stop_tx = None;
            self.seek_tx = None;
            self.pause_tx = None;
            self.done_rx = None;
            self.position_ms = None;
            self.paused = false;
            self.levels.reset();
            self.preload_rx = self.loaded_path.as_ref().map(|path| preload(path.clone()));
        }
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        PlayerSnapshot {
            id: self.id,
            state: self.state(),
            loaded_path: self.loaded_path.clone(),
            position: self.position(),
            duration: self.duration,
        }
    }

    pub fn is_active(&self) -> bool {
        self.stop_tx.is_some()
    }

    pub fn is_playing(&self) -> bool {
        self.stop_tx.is_some() && !self.paused
    }

    pub fn levels(&self) -> (f32, f32) {
        self.levels.read()
    }

    fn state(&self) -> PlayerState {
        if self.stop_tx.is_some() && self.paused {
            PlayerState::Paused
        } else if self.stop_tx.is_some() {
            PlayerState::Playing
        } else if self.loaded_path.is_some() {
            PlayerState::Loaded
        } else {
            PlayerState::Empty
        }
    }

    fn position(&self) -> Duration {
        self.position_ms
            .as_ref()
            .map(|pos| Duration::from_millis(pos.load(Ordering::Relaxed)))
            .unwrap_or_default()
    }

    fn stop_thread(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        self.seek_tx = None;
        self.pause_tx = None;
        self.done_rx = None;
        self.position_ms = None;
        self.paused = false;
        self.levels.reset();
    }
}

#[derive(Default)]
pub struct AudioLevels {
    left: AtomicU32,
    right: AtomicU32,
}

impl AudioLevels {
    fn store(&self, left: f32, right: f32) {
        self.left
            .store(left.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        self.right
            .store(right.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    fn reset(&self) {
        self.store(0.0, 0.0);
    }

    fn read(&self) -> (f32, f32) {
        (
            f32::from_bits(self.left.load(Ordering::Relaxed)),
            f32::from_bits(self.right.load(Ordering::Relaxed)),
        )
    }
}

/// Résultat du pré-chargement : décodeur prêt + buffer pré-rempli avec ~0.5 s d'audio.
pub struct Preloaded {
    samples: Vec<f32>,
    format: Box<dyn symphonia::core::formats::FormatReader>,
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    track_id: u32,
    source_sample_rate: u32,
}

struct AudioChunk {
    generation: u64,
    samples: Vec<f32>,
}

enum SeekRequest {
    Relative(i64),
    Absolute(Duration),
}

/// Lit uniquement les métadonnées pour obtenir la durée totale du fichier.
pub fn read_duration(path: &std::path::Path) -> Option<std::time::Duration> {
    let file = std::fs::File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;
    let track = probed.format.default_track()?;
    let n_frames = track.codec_params.n_frames?;
    let sample_rate = track.codec_params.sample_rate? as f64;
    Some(std::time::Duration::from_secs_f64(
        n_frames as f64 / sample_rate,
    ))
}

/// Démarre le pré-chargement en arrière-plan dès la sélection du fichier.
/// Retourne un Receiver : `try_recv()` donne le résultat quand il est prêt.
pub fn preload(path: PathBuf) -> mpsc::Receiver<Option<Preloaded>> {
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _ = tx.send(do_preload(path).ok());
    });
    rx
}

fn do_preload(path: PathBuf) -> Result<Preloaded, Box<dyn std::error::Error + Send + Sync>> {
    let file = std::fs::File::open(&path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = symphonia::default::get_probe().format(
        &Hint::new(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;
    let track = format.default_track().ok_or("No audio track")?;
    let track_id = track.id;
    let source_sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    // Déterminer la config cpal pour mixer correctement dès le pré-chargement
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No audio output device available")?;
    let out_channels = config_for(&device, source_sample_rate)?.channels() as usize;

    let mut samples = Vec::new();
    let prefill_target = source_sample_rate as usize * out_channels / 2; // 0.5 s

    loop {
        if samples.len() >= prefill_target {
            break;
        }
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => samples.extend(decoded_to_samples(decoded, out_channels)),
            Err(_) => continue,
        }
    }

    Ok(Preloaded {
        samples,
        format,
        decoder,
        track_id,
        source_sample_rate,
    })
}

/// Lance la lecture. Si `preloaded` est fourni, le buffer est déjà prêt → Play instantané.
/// Retourne `(stop_tx, seek_tx, pause_tx, position_ms, done_rx)`.
/// - `stop_tx`  : envoyer `()` arrête et remet à zéro
/// - `pause_tx` : envoyer `true` pour pause, `false` pour reprendre
fn play(
    path: PathBuf,
    preloaded: Option<Preloaded>,
    levels: Arc<AudioLevels>,
) -> (
    Sender<()>,
    Sender<SeekRequest>,
    Sender<bool>,
    Arc<AtomicU64>,
    mpsc::Receiver<()>,
) {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (seek_tx, seek_rx) = mpsc::channel::<SeekRequest>();
    let (pause_tx, pause_rx) = mpsc::channel::<bool>();
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let position_ms = Arc::new(AtomicU64::new(0));
    let position_ms_thread = Arc::clone(&position_ms);
    std::thread::spawn(move || {
        if let Err(e) = run(
            path,
            preloaded,
            stop_rx,
            seek_rx,
            pause_rx,
            position_ms_thread,
            levels,
        ) {
            eprintln!("Audio error: {e}");
        }
        let _ = done_tx.send(());
    });
    (stop_tx, seek_tx, pause_tx, position_ms, done_rx)
}

fn run(
    path: PathBuf,
    preloaded: Option<Preloaded>,
    stop_rx: mpsc::Receiver<()>,
    seek_rx: mpsc::Receiver<SeekRequest>,
    pause_rx: mpsc::Receiver<bool>,
    position_ms: Arc<AtomicU64>,
    levels: Arc<AudioLevels>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Récupérer l'état symphonia — depuis le pré-chargement ou en ouvrant le fichier
    let (mut format, mut decoder, track_id, source_sample_rate, preloaded_samples) = match preloaded
    {
        Some(p) => (
            p.format,
            p.decoder,
            p.track_id,
            p.source_sample_rate,
            p.samples,
        ),
        None => {
            let file = std::fs::File::open(&path)?;
            let mss = MediaSourceStream::new(Box::new(file), Default::default());
            let probed = symphonia::default::get_probe().format(
                &Hint::new(),
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )?;
            let fmt = probed.format;
            let track = fmt.default_track().ok_or("No audio track")?;
            let track_id = track.id;
            let sr = track.codec_params.sample_rate.unwrap_or(44100);
            let dec = symphonia::default::get_codecs()
                .make(&track.codec_params, &DecoderOptions::default())?;
            (fmt, dec, track_id, sr, Vec::new())
        }
    };

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No audio output device available")?;
    let config = config_for(&device, source_sample_rate)?;
    let out_channels = config.channels() as usize;
    let (chunk_tx, chunk_rx) = mpsc::sync_channel::<AudioChunk>(64);
    let buffered_samples = Arc::new(AtomicUsize::new(preloaded_samples.len()));
    let playback_generation = Arc::new(AtomicU64::new(0));

    let levels_cb = Arc::clone(&levels);
    let buffered_samples_cb = Arc::clone(&buffered_samples);
    let playback_generation_cb = Arc::clone(&playback_generation);
    let mut current_chunk = preloaded_samples;
    let mut current_offset = 0;
    let mut callback_generation = 0;
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            fill_output_from_chunks(
                data,
                &chunk_rx,
                &buffered_samples_cb,
                &playback_generation_cb,
                &mut current_chunk,
                &mut current_offset,
                &mut callback_generation,
            );
            update_levels(&levels_cb, data, out_channels);
        },
        |err| eprintln!("CPAL error: {err}"),
        None,
    )?;
    // Le buffer contient déjà ~0.5 s si pré-chargé : premier son immédiat
    stream.play()?;

    let mut current_ts: u64 = 0;
    let mut paused = false;

    loop {
        if stop_rx.try_recv().is_ok() {
            return Ok(());
        }

        if let Ok(should_pause) = pause_rx.try_recv() {
            if should_pause && !paused {
                stream.pause().ok();
                levels.reset();
                paused = true;
            } else if !should_pause && paused {
                stream.play().ok();
                paused = false;
            }
        }

        if paused {
            std::thread::sleep(std::time::Duration::from_millis(20));
            continue;
        }

        if let Ok(seek) = seek_rx.try_recv() {
            let audible_secs = position_ms.load(Ordering::Relaxed) as f64 / 1000.0;
            let target_secs = match seek {
                SeekRequest::Relative(offset_ms) => {
                    (audible_secs + offset_ms as f64 / 1000.0).max(0.0)
                }
                SeekRequest::Absolute(position) => position.as_secs_f64(),
            };
            let target_time = Time {
                seconds: target_secs as u64,
                frac: target_secs.fract(),
            };
            if format
                .seek(
                    SeekMode::Coarse,
                    SeekTo::Time {
                        time: target_time,
                        track_id: Some(track_id),
                    },
                )
                .is_ok()
            {
                decoder.reset();
                current_ts = (target_secs * source_sample_rate as f64) as u64;
                position_ms.store((target_secs * 1000.0).max(0.0) as u64, Ordering::Relaxed);
                buffered_samples.store(0, Ordering::Relaxed);
                playback_generation.fetch_add(1, Ordering::Relaxed);
            }
            continue;
        }

        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }
        current_ts = packet.ts();

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::DecodeError(msg)) => {
                eprintln!("Decode warning: {msg}");
                continue;
            }
            Err(e) => return Err(Box::new(e)),
        };

        let samples = decoded_to_samples(decoded, out_channels);
        let sample_count = samples.len();
        let generation = playback_generation.load(Ordering::Relaxed);
        let chunk = AudioChunk {
            generation,
            samples,
        };

        if !send_audio_chunk(&chunk_tx, chunk, &stop_rx, &buffered_samples, sample_count) {
            return Ok(());
        }

        // Position affichée = position décodée − contenu du buffer (audio pas encore joué)
        update_position_from_buffer(
            &position_ms,
            current_ts,
            source_sample_rate,
            out_channels,
            buffered_samples.load(Ordering::Relaxed),
        );
    }

    // Arrêt explicite : vider immédiatement
    if stop_rx.try_recv().is_ok() {
        return Ok(());
    }

    // Fin naturelle : attendre que le callback cpal ait consommé tous les blocs envoyés.
    loop {
        if stop_rx.try_recv().is_ok() {
            return Ok(());
        }
        let remaining = buffered_samples.load(Ordering::Relaxed);
        if remaining == 0 {
            break;
        }
        update_position_from_buffer(
            &position_ms,
            current_ts,
            source_sample_rate,
            out_channels,
            remaining,
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Laisser le ring buffer hardware (CoreAudio / WASAPI) terminer sa lecture
    // avant de dropper le stream — sinon les dernières ~100 ms sont coupées
    for _ in 0..20 {
        if stop_rx.try_recv().is_ok() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}

fn fill_output_from_chunks(
    data: &mut [f32],
    chunk_rx: &mpsc::Receiver<AudioChunk>,
    buffered_samples: &AtomicUsize,
    playback_generation: &AtomicU64,
    current_chunk: &mut Vec<f32>,
    current_offset: &mut usize,
    callback_generation: &mut u64,
) {
    let active_generation = playback_generation.load(Ordering::Relaxed);
    if *callback_generation != active_generation {
        current_chunk.clear();
        *current_offset = 0;
        *callback_generation = active_generation;
    }

    let mut written = 0;
    while written < data.len() {
        if *current_offset >= current_chunk.len() {
            current_chunk.clear();
            *current_offset = 0;

            match chunk_rx.try_recv() {
                Ok(chunk) if chunk.generation == active_generation => {
                    *current_chunk = chunk.samples;
                }
                Ok(_) => continue,
                Err(_) => {
                    data[written..].fill(0.0);
                    break;
                }
            }
        }

        let available = current_chunk.len().saturating_sub(*current_offset);
        let to_copy = available.min(data.len() - written);
        if to_copy == 0 {
            continue;
        }

        let src_end = *current_offset + to_copy;
        let dst_end = written + to_copy;
        data[written..dst_end].copy_from_slice(&current_chunk[*current_offset..src_end]);
        decrement_buffered_samples(buffered_samples, to_copy);
        *current_offset = src_end;
        written = dst_end;
    }
}

fn decrement_buffered_samples(buffered_samples: &AtomicUsize, consumed: usize) {
    let _ = buffered_samples.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_sub(consumed))
    });
}

fn send_audio_chunk(
    chunk_tx: &SyncSender<AudioChunk>,
    mut chunk: AudioChunk,
    stop_rx: &mpsc::Receiver<()>,
    buffered_samples: &AtomicUsize,
    sample_count: usize,
) -> bool {
    loop {
        buffered_samples.fetch_add(sample_count, Ordering::Relaxed);
        match chunk_tx.try_send(chunk) {
            Ok(()) => return true,
            Err(TrySendError::Full(returned_chunk)) => {
                decrement_buffered_samples(buffered_samples, sample_count);
                if stop_rx.try_recv().is_ok() {
                    return false;
                }
                chunk = returned_chunk;
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(TrySendError::Disconnected(_)) => {
                decrement_buffered_samples(buffered_samples, sample_count);
                return false;
            }
        }
    }
}

fn update_position_from_buffer(
    position_ms: &AtomicU64,
    current_ts: u64,
    source_sample_rate: u32,
    out_channels: usize,
    buffered_sample_count: usize,
) {
    let buf_ms = buffered_sample_count as u64 * 1000
        / (source_sample_rate as u64 * out_channels.max(1) as u64);
    let decode_ms = current_ts * 1000 / source_sample_rate as u64;
    position_ms.store(decode_ms.saturating_sub(buf_ms), Ordering::Relaxed);
}

fn update_levels(levels: &AudioLevels, data: &[f32], channels: usize) {
    if channels == 0 {
        levels.reset();
        return;
    }

    let mut left = 0.0_f32;
    let mut right = 0.0_f32;

    for frame in data.chunks(channels) {
        let l = frame.first().copied().unwrap_or(0.0).abs();
        let r = frame.get(1).copied().unwrap_or(l).abs();
        left = left.max(l);
        right = right.max(r);
    }

    levels.store(left, right);
}

fn decoded_to_samples(decoded: AudioBufferRef<'_>, out_channels: usize) -> Vec<f32> {
    let spec = *decoded.spec();
    let capacity = decoded.capacity() as u64;
    let mut sample_buf = SampleBuffer::<f32>::new(capacity, spec);
    sample_buf.copy_interleaved_ref(decoded);
    let samples = sample_buf.samples();
    let src_ch = spec.channels.count();

    if src_ch == out_channels {
        samples.to_vec()
    } else if src_ch == 1 {
        let mut out = Vec::with_capacity(samples.len() * out_channels);
        for &s in samples {
            for _ in 0..out_channels {
                out.push(s);
            }
        }
        out
    } else {
        let frame_count = samples.len() / src_ch;
        let mut out = Vec::with_capacity(frame_count * out_channels);
        for frame in samples.chunks(src_ch) {
            for i in 0..out_channels {
                out.push(*frame.get(i % src_ch).unwrap_or(&0.0));
            }
        }
        out
    }
}

fn config_for(
    device: &cpal::Device,
    sample_rate: u32,
) -> Result<SupportedStreamConfig, Box<dyn std::error::Error + Send + Sync>> {
    for cfg in device.supported_output_configs()? {
        if cfg.min_sample_rate().0 <= sample_rate && sample_rate <= cfg.max_sample_rate().0 {
            return Ok(cfg.with_sample_rate(SampleRate(sample_rate)));
        }
    }
    eprintln!(
        "Warning: {} Hz is not supported, falling back to the default config",
        sample_rate
    );
    Ok(device.default_output_config()?)
}
