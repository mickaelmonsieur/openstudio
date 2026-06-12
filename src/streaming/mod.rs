use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::audio::BroadcastFrame;
use crate::db;

#[cfg(openstudio_has_lame)]
mod lame;
#[cfg(openstudio_has_shout)]
mod shout;

pub struct StreamingHandle {
    stop_tx: mpsc::Sender<()>,
    config: Arc<Mutex<StreamingConfig>>,
    status: Arc<Mutex<String>>,
    metadata_title: Arc<Mutex<Option<String>>>,
}

impl StreamingHandle {
    pub fn stop(&self) {
        let _ = self.stop_tx.send(());
    }

    pub fn update_config(&self, config: StreamingConfig) {
        if let Ok(mut current) = self.config.lock() {
            *current = config;
        }
    }

    pub fn status(&self) -> String {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_else(|_| String::from("Status unavailable"))
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
    pub host: String,
    pub port: i32,
    pub password: String,
    pub mountpoint: String,
    pub reconnect_seconds: i32,
}

impl From<&db::AppConfig> for StreamingConfig {
    fn from(cfg: &db::AppConfig) -> Self {
        Self {
            bitrate_kbps: cfg.encoder_bitrate.clamp(8, 320),
            sample_rate: cfg.encoder_sample_rate.clamp(8_000, 48_000),
            channels: cfg.encoder_channels.clamp(1, 2),
            host: cfg.encoder_server_host.clone(),
            port: cfg.encoder_server_port.clamp(1, 65_535),
            password: cfg.encoder_password.clone(),
            mountpoint: normalized_mountpoint(&cfg.encoder_mountpoint),
            reconnect_seconds: cfg.encoder_reconnect_seconds.clamp(1, 3_600),
        }
    }
}

pub fn start(
    config: StreamingConfig,
    broadcast_rx: mpsc::Receiver<BroadcastFrame>,
) -> StreamingHandle {
    let (stop_tx, stop_rx) = mpsc::channel();
    let config = Arc::new(Mutex::new(config));
    let status = Arc::new(Mutex::new(String::from("Starting")));
    let metadata_title = Arc::new(Mutex::new(None));
    let worker_config = Arc::clone(&config);
    let worker_status = Arc::clone(&status);
    let worker_metadata_title = Arc::clone(&metadata_title);
    thread::spawn(move || {
        run(
            worker_config,
            worker_status,
            worker_metadata_title,
            broadcast_rx,
            stop_rx,
        )
    });
    StreamingHandle {
        stop_tx,
        config,
        status,
        metadata_title,
    }
}

fn run(
    config: Arc<Mutex<StreamingConfig>>,
    status: Arc<Mutex<String>>,
    metadata_title: Arc<Mutex<Option<String>>>,
    broadcast_rx: mpsc::Receiver<BroadcastFrame>,
    stop_rx: mpsc::Receiver<()>,
) {
    #[cfg(all(openstudio_has_lame, openstudio_has_shout))]
    {
        run_lame_shout(config, status, metadata_title, broadcast_rx, stop_rx);
    }

    #[cfg(not(all(openstudio_has_lame, openstudio_has_shout)))]
    {
        run_noop(config, status, metadata_title, broadcast_rx, stop_rx);
    }
}

#[cfg(all(openstudio_has_lame, openstudio_has_shout))]
fn run_lame_shout(
    config: Arc<Mutex<StreamingConfig>>,
    status: Arc<Mutex<String>>,
    metadata_title: Arc<Mutex<Option<String>>>,
    broadcast_rx: mpsc::Receiver<BroadcastFrame>,
    stop_rx: mpsc::Receiver<()>,
) {
    let mut sent_metadata_title: Option<String> = None;

    loop {
        if stop_rx.try_recv().is_ok() {
            return;
        }

        let current_config = config_snapshot(&config);
        set_status(
            &status,
            format!("Connecting to {}", current_config.mountpoint),
        );
        let mut encoder = match lame::LameMp3Encoder::new(&current_config) {
            Ok(encoder) => encoder,
            Err(error) => {
                set_status(&status, format!("Encoder error: {error}"));
                sleep_reconnect(&current_config, &stop_rx);
                continue;
            }
        };
        let mut client = match shout::IcecastClient::connect(&current_config) {
            Ok(client) => client,
            Err(error) => {
                set_status(&status, format!("Icecast connection error: {error}"));
                sleep_reconnect(&current_config, &stop_rx);
                continue;
            }
        };
        set_status(&status, String::from("Connected"));
        sent_metadata_title =
            send_pending_metadata(&mut client, &metadata_title, sent_metadata_title, &status);

        loop {
            if stop_rx.try_recv().is_ok() {
                return;
            }
            let frame = match broadcast_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(frame) => frame,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            };
            sent_metadata_title =
                send_pending_metadata(&mut client, &metadata_title, sent_metadata_title, &status);
            let encoded = match encoder.encode(&frame) {
                Ok(encoded) => encoded,
                Err(error) => {
                    set_status(&status, format!("MP3 encode error: {error}"));
                    break;
                }
            };
            if encoded.is_empty() {
                continue;
            }
            if let Err(error) = client.send(&encoded) {
                set_status(&status, format!("Icecast send error: {error}"));
                break;
            }
        }

        sleep_reconnect(&config_snapshot(&config), &stop_rx);
    }
}

#[cfg(not(all(openstudio_has_lame, openstudio_has_shout)))]
fn run_noop(
    config: Arc<Mutex<StreamingConfig>>,
    status: Arc<Mutex<String>>,
    _metadata_title: Arc<Mutex<Option<String>>>,
    broadcast_rx: mpsc::Receiver<BroadcastFrame>,
    stop_rx: mpsc::Receiver<()>,
) {
    let config = config_snapshot(&config);
    set_status(
        &status,
        format!(
            "Unavailable: libmp3lame/libshout not found at build time ({}:{}{})",
            config.host, config.port, config.mountpoint
        ),
    );
    loop {
        if stop_rx.try_recv().is_ok() {
            return;
        }
        match broadcast_rx.recv_timeout(Duration::from_millis(250)) {
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

#[cfg(all(openstudio_has_lame, openstudio_has_shout))]
fn send_pending_metadata(
    client: &mut shout::IcecastClient,
    metadata_title: &Arc<Mutex<Option<String>>>,
    last_sent: Option<String>,
    status: &Arc<Mutex<String>>,
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
            set_status(status, format!("Icecast metadata error: {error}"));
            last_sent
        }
    }
}

fn set_status(status: &Arc<Mutex<String>>, value: impl Into<String>) {
    if let Ok(mut status) = status.lock() {
        *status = value.into();
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
            host: String::from("openstudio.entrypoint.belstream.net"),
            port: 80,
            password: String::new(),
            mountpoint: String::from("/live"),
            reconnect_seconds: 10,
        })
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
