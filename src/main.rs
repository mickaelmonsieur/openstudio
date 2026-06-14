#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_audio_config;
mod app_aux;
mod app_constants;
mod app_helpers;
mod app_instant;
mod app_paths;
mod app_queue;
mod app_rest;
mod app_search;
mod app_stream_encoder;
mod app_streaming;
mod app_time;
mod audio;
mod db;
mod rest;
mod streaming;
mod ui;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::app_audio_config::find_compressor_preset;
use crate::app_constants::{
    ANY_CATEGORY, ANY_GENRE, ANY_SUBCATEGORY, METER_TICK_MS, PREVIEW_PLAYER_ID, QUEUE_PLAYER_IDS,
    SEARCH_PAGE_SIZE,
};
use crate::app_helpers::{
    audio_peak_to_meter, login_input_id, main_window_settings, pass_input_id,
    picker_window_settings, smooth_meter,
};
use crate::app_paths::{
    db_config_path, db_config_save_path, migrations_dir, pg_quote_ident, DEFAULT_PSQL_PATH,
};
use crate::app_time::{current_date, current_hour};
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::{window, Subscription, Task, Theme};

fn main() -> iced::Result {
    iced::daemon(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .run_with(App::new)
}

// ── State ─────────────────────────────────────────────────────────────────────

struct App {
    windows: HashMap<window::Id, WindowKind>,
    main_window: Option<window::Id>,
    db: Option<db::SharedDatabase>,
    audio: audio::AudioManager,
    status: String,
    auto_mix_status: String,
    queue_entries: Vec<db::QueueEntry>,
    queue_player_entries: HashMap<audio::PlayerId, db::QueueEntry>,
    active_queue_play_logs: HashMap<audio::PlayerId, ActiveQueuePlayLog>,
    preloaded_queue_entry: Option<PreloadedQueueEntry>,
    current_queue_entry: Option<db::QueueEntry>,
    track_end_at: Option<std::time::SystemTime>,
    current_queue_player_id: audio::PlayerId,
    audio_devices: Vec<audio::AudioDeviceInfo>,
    audio_input_devices: Vec<audio::AudioDeviceInfo>,
    selected_queue_index: Option<usize>,
    autodj_enabled: bool,
    deck_soft_stopping: bool,
    previewing_queue_id: Option<i32>,
    search_tracks: Vec<db::SearchTrack>,
    search_total_rows: usize,
    search_categories: Vec<db::FilterOption>,
    search_subcategories: Vec<db::FilterOption>,
    search_genres: Vec<db::FilterOption>,
    instant_view: InstantView,
    search_query: String,
    search_category: db::FilterOption,
    search_subcategory: db::FilterOption,
    search_genre: db::FilterOption,
    search_page_start: usize,
    selected_search_track_id: Option<i32>,
    current_hour: String,
    current_date: String,
    queue_meter_left: f32,
    queue_meter_right: f32,
    instant_pages: Vec<InstantPage>,
    active_instant_page: usize,
    instant_slots: Vec<Option<LoadedTrack>>,
    active_instant_slot: Option<usize>,
    instant_context_slot: Option<usize>,
    aux_slots: Vec<Option<LoadedTrack>>,
    aux_loops: Vec<bool>,
    instant_loops: Vec<bool>,
    rest_rx: Arc<Mutex<Option<std::sync::mpsc::Receiver<rest::RestCommand>>>>,
    rest_shutdown_tx: Option<std::sync::mpsc::Sender<()>>,
    streaming_handle: Option<streaming::StreamingHandle>,
    station_name: String,
    app_config: db::AppConfig,
    timezone_options: Vec<String>,
    dialog: Option<Dialog>,
    is_locked: bool,
    current_user_login: String,
    current_user_role: i16,
    login_pending: Option<PendingLogin>,
}

impl Default for App {
    fn default() -> Self {
        let search_category = db::FilterOption::all(ANY_CATEGORY);
        let search_subcategory = db::FilterOption::all(ANY_SUBCATEGORY);
        let search_genre = db::FilterOption::all(ANY_GENRE);
        let mut search_categories = vec![search_category.clone()];
        let mut search_subcategories = vec![search_subcategory.clone()];
        let mut search_genres = vec![search_genre.clone()];
        let mut search_tracks = Vec::new();
        let mut search_total_rows = 0;
        let mut queue_entries = Vec::new();
        let mut app_config = db::AppConfig::default();
        let mut station_name = String::from("OpenStudio");
        let mut timezone_options = Vec::new();
        let (rest_tx, rest_rx) = std::sync::mpsc::channel();
        let rest_shutdown_tx = rest::start_server(rest_tx);

        let (db, status) = match db::Database::connect_from_file(&db_config_path()) {
            Ok(db) => {
                let mut warnings = Vec::new();

                match db.load_config() {
                    Ok(cfg) => app_config = cfg,
                    Err(error) => warnings.push(format!("config: {error}")),
                }

                match db.search_tracks_page("", None, None, None, 0, SEARCH_PAGE_SIZE) {
                    Ok((tracks, total)) => {
                        search_tracks = tracks;
                        search_total_rows = total;
                    }
                    Err(error) => warnings.push(format!("tracks: {error}")),
                }

                match db.queue_entries(&app_config.timezone) {
                    Ok(entries) => queue_entries = entries,
                    Err(error) => warnings.push(format!("queue: {error}")),
                }

                match db.station_name() {
                    Ok(Some(name)) if !name.trim().is_empty() => station_name = name,
                    Ok(_) => {}
                    Err(error) => warnings.push(format!("station: {error}")),
                }

                match db.timezones() {
                    Ok(options) => timezone_options = options,
                    Err(error) => warnings.push(format!("timezones: {error}")),
                }

                match db.categories() {
                    Ok(options) => search_categories.extend(options),
                    Err(error) => warnings.push(format!("categories: {error}")),
                }

                match db.subcategories() {
                    Ok(options) => search_subcategories.extend(options),
                    Err(error) => warnings.push(format!("subcategories: {error}")),
                }

                match db.genres() {
                    Ok(options) => search_genres.extend(options),
                    Err(error) => warnings.push(format!("genres: {error}")),
                }

                let status = if warnings.is_empty() {
                    String::from("Connected")
                } else {
                    format!("Connected (partial: {})", warnings.join(" | "))
                };

                (Some(db), status)
            }
            Err(error) => (None, format!("Disconnected ({error})")),
        };

        let mut app = Self {
            windows: HashMap::new(),
            main_window: None,
            db,
            audio: audio::AudioManager::new(),
            status,
            auto_mix_status: if app_config.auto_mix_on_start {
                String::from("Waiting")
            } else {
                String::from("Disabled")
            },
            queue_entries,
            queue_player_entries: HashMap::new(),
            active_queue_play_logs: HashMap::new(),
            preloaded_queue_entry: None,
            current_queue_entry: None,
            track_end_at: None,
            current_queue_player_id: audio::PlayerId::QueueA,
            audio_devices: audio::list_output_devices(),
            audio_input_devices: audio::list_input_devices(),
            selected_queue_index: None,
            autodj_enabled: app_config.auto_mix_on_start,
            deck_soft_stopping: false,
            previewing_queue_id: None,
            search_tracks,
            search_total_rows,
            search_categories,
            search_subcategories,
            search_genres,
            instant_view: InstantView::InstantPlayers,
            search_query: String::new(),
            search_category,
            search_subcategory,
            search_genre,
            search_page_start: 0,
            selected_search_track_id: None,
            current_hour: current_hour(),
            current_date: current_date(),
            queue_meter_left: 0.0,
            queue_meter_right: 0.0,
            instant_pages: vec![InstantPage::default()],
            active_instant_page: 0,
            instant_slots: vec![None; 10],
            active_instant_slot: None,
            instant_context_slot: None,
            aux_slots: vec![None; 3],
            aux_loops: vec![false; 3],
            instant_loops: vec![false; 10],
            rest_rx: Arc::new(Mutex::new(Some(rest_rx))),
            rest_shutdown_tx: Some(rest_shutdown_tx),
            streaming_handle: None,
            station_name,
            is_locked: app_config.start_locked,
            app_config,
            timezone_options,
            dialog: None,
            current_user_login: String::from("user"),
            current_user_role: 0,
            login_pending: None,
        };
        app.ensure_configured_timezone_option();
        app.apply_audio_processing_config(&app.app_config.clone());
        app.apply_audio_device_config(&app.app_config.clone());
        app.sync_streaming_encoder();
        app.load_instant_pages_from_db();
        let auto_mix_status = app.auto_mix_status.clone();
        app.log_auto_mix_status(&auto_mix_status);
        app
    }
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();
        app.apply_startup_playback_config();

        let (window_id, open) = window::open(main_window_settings());
        app.main_window = Some(window_id);
        app.windows.insert(window_id, WindowKind::Main);
        (app, open.map(|_| Message::NoOp))
    }
}

#[derive(Debug, Clone)]
enum WindowKind {
    Main,
    TrackPicker(TrackPickerState),
}

#[derive(Debug, Clone)]
pub(crate) struct TrackPickerState {
    pub(crate) target: PickerTarget,
    pub(crate) search_query: String,
    pub(crate) search_category: db::FilterOption,
    pub(crate) search_subcategory: db::FilterOption,
    pub(crate) search_genre: db::FilterOption,
    pub(crate) page_start: usize,
    pub(crate) tracks: Vec<db::SearchTrack>,
    pub(crate) total_rows: usize,
    pub(crate) selected_track_id: Option<i32>,
    pub(crate) last_click: Option<(i32, std::time::Instant)>,
}

#[derive(Debug, Clone)]
struct PreloadedQueueEntry {
    player_id: audio::PlayerId,
    entry: db::QueueEntry,
}

#[derive(Debug, Clone)]
struct ActiveQueuePlayLog {
    track_id: i32,
    cue_in: std::time::Duration,
    cue_out: std::time::Duration,
    duration: std::time::Duration,
}

impl ActiveQueuePlayLog {
    fn audible_played_duration(&self, position: std::time::Duration) -> std::time::Duration {
        position.saturating_sub(self.cue_in)
    }

    fn expected_audible_duration(&self) -> std::time::Duration {
        let cue_out = if self.cue_out > self.cue_in {
            self.cue_out
        } else {
            self.duration
        };
        cue_out.saturating_sub(self.cue_in)
    }

    fn was_read_to_end(&self, played_duration: std::time::Duration) -> bool {
        let expected = self.expected_audible_duration();
        !expected.is_zero()
            && played_duration.saturating_add(std::time::Duration::from_millis(1500)) >= expected
    }
}

impl TrackPickerState {
    fn new(target: PickerTarget) -> Self {
        Self {
            target,
            search_query: String::new(),
            search_category: db::FilterOption::all(ANY_CATEGORY),
            search_subcategory: db::FilterOption::all(ANY_SUBCATEGORY),
            search_genre: db::FilterOption::all(ANY_GENRE),
            page_start: 0,
            tracks: Vec::new(),
            total_rows: 0,
            selected_track_id: None,
            last_click: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PickerTarget {
    Instant(usize),
    Aux(usize),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct LoadedTrack {
    pub(crate) id: i32,
    pub(crate) artist: String,
    pub(crate) title: String,
    pub(crate) duration: std::time::Duration,
    pub(crate) cue_in: std::time::Duration,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Clone)]
struct InstantPage {
    id: Option<i32>,
    name: String,
}

impl Default for InstantPage {
    fn default() -> Self {
        Self {
            id: None,
            name: String::from("Default"),
        }
    }
}

#[derive(Debug, Clone)]
enum PendingLogin {
    ConfigOpen,
}

#[derive(Debug, Clone)]
enum Dialog {
    About,
    SaveInstantPage {
        name: String,
    },
    EditConfig {
        auto_mix_on_start: bool,
        auto_play_on_start: bool,
        start_locked: bool,
        preload: String,
        fade_out_duration_ms: String,
        stop_fade_duration_ms: String,
        timezone: String,
        device_deck_id: String,
        device_instant_id: String,
        device_aux_id: String,
        device_preview_id: String,
        encoder_input_device_id: String,
    },
    EditDbConfig {
        host: String,
        port: String,
        database: String,
        user: String,
        password: String,
        psql_path: String,
        connection_status: Option<Result<String, String>>,
        create_status: Option<Result<String, String>>,
        delete_status: Option<Result<String, String>>,
        delete_confirm: bool,
    },
    AudioProcessing {
        processing_bypassed: bool,
        input_volume: f32,
        compressor_mode: String,
        compressor_preset: String,
        attack: String,
        ratio: String,
        threshold: String,
        gain: String,
        release: String,
        eq_enabled: bool,
        eq_gains: Vec<f32>,
        agc_preset: String,
    },
    StreamEncoder {
        enabled: bool,
        bitrate: String,
        sample_rate: String,
        channels: String,
        encoder_type: String,
        server_host: String,
        server_port: String,
        password: String,
        mountpoint: String,
        reconnect_seconds: String,
        error: Option<String>,
    },
    Login {
        login: String,
        password: String,
        error: Option<String>,
        focus_index: usize,
    },
    ConfirmClose {
        window_id: window::Id,
    },
}

#[derive(Debug, Clone)]
enum LoginField {
    Login,
    Password,
}

#[derive(Debug, Clone)]
enum DbField {
    Host,
    Port,
    Database,
    User,
    Password,
    PsqlPath,
}

#[derive(Debug, Clone)]
enum ConfigField {
    AutoMixOnStart,
    AutoPlayOnStart,
    StartLocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceTarget {
    Deck,
    Instant,
    Aux,
    Preview,
    StreamInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstantView {
    Search,
    InstantPlayers,
}

#[derive(Debug, Clone)]
enum Message {
    NoOp,
    WindowClosed(window::Id),
    Stop,
    TogglePlay,
    Restart,
    PollDone,
    Seek(i64),
    Player(audio::PlayerId, audio::PlayerCommand),
    SearchPreviewPlay,
    ShowSearch,
    ShowInstantPlayers,
    SearchChanged(String),
    CategorySelected(db::FilterOption),
    SubcategorySelected(db::FilterOption),
    GenreSelected(db::FilterOption),
    SearchRowSelected(i32),
    SearchFirstPage,
    SearchPreviousPage,
    SearchNextPage,
    SearchLastPage,
    OpenTrackPicker(PickerTarget),
    PickerSearchChanged(window::Id, String),
    PickerCategorySelected(window::Id, db::FilterOption),
    PickerSubcategorySelected(window::Id, db::FilterOption),
    PickerGenreSelected(window::Id, db::FilterOption),
    PickerRowPressed(window::Id, i32),
    PickerFirstPage(window::Id),
    PickerPreviousPage(window::Id),
    PickerNextPage(window::Id),
    PickerLastPage(window::Id),
    PickerPreviewPlay(window::Id),
    InstantSlotPressed(usize),
    InstantSlotRightClick(usize),
    InstantSlotLoad(usize),
    InstantSlotClear(usize),
    InstantStop,
    InstantSave,
    InstantSaveNameChanged(String),
    InstantSaveConfirm,
    DialogCancel,
    AboutOpen,
    InstantNewPage,
    InstantDeletePage,
    InstantPreviousPage,
    InstantNextPage,
    ToggleAutoDj,
    Rest(rest::RestCommand),
    QueuePreviewToggle(i32),
    QueueMoveUp,
    QueueMoveTop,
    QueueMoveDown,
    QueueMoveBottom,
    QueueRowSelected(usize),
    QueuePlayNow(usize),
    QueueInsertTrack,
    QueueReplaceTrack,
    QueueRemoveEntry,
    QueueClearAll,
    QueueReload,
    AuxPlay(usize),
    AuxStop(usize),
    AuxToggleLoop(usize),
    MeterTick,
    ClockTick,
    DbConfigOpen,
    DbConfigFieldChanged(DbField, String),
    DbConfigSave,
    DbConfigConnected(Result<db::SharedDatabase, String>),
    DbConfigTestConnection,
    DbConfigTestResult(Result<String, String>),
    DbConfigCreateDatabase,
    DbConfigCreateResult(Result<String, String>),
    DbConfigAskDeleteDatabase,
    DbConfigCancelDeleteDatabase,
    DbConfigDeleteDatabase,
    DbConfigDeleteResult(Result<String, String>),
    ConfigOpen,
    AudioProcessingOpen,
    AudioProcessingBypassChanged(bool),
    AudioProcessingInputVolumeChanged(f32),
    AudioProcessingModeChanged(String),
    AudioProcessingPresetChanged(String),
    AudioProcessingAttackChanged(String),
    AudioProcessingRatioChanged(String),
    AudioProcessingThresholdChanged(String),
    AudioProcessingGainChanged(String),
    AudioProcessingReleaseChanged(String),
    AudioProcessingEqEnabledChanged(bool),
    AudioProcessingEqGainChanged(usize, f32),
    AudioProcessingAgcPresetChanged(String),
    AudioProcessingSave,
    StreamEncoderOpen,
    StreamEncoderEnabledChanged(bool),
    StreamEncoderBitrateChanged(String),
    StreamEncoderSampleRateChanged(String),
    StreamEncoderTypeChanged(String),
    StreamEncoderChannelsChanged(String),
    StreamEncoderPasswordChanged(String),
    StreamEncoderMountpointChanged(String),
    StreamEncoderReconnectChanged(String),
    StreamEncoderSave,
    ConfigToggle(ConfigField),
    ConfigPreloadChanged(String),
    ConfigFadeOutDurationChanged(String),
    ConfigStopFadeDurationChanged(String),
    ConfigTimezoneChanged(String),
    ConfigDeviceChanged(DeviceTarget, String),
    ConfigSave,
    ConfigSaved(Result<(), String>),
    CloseRequested(window::Id),
    ConfirmQuit,
    LockToggle,
    LoginFieldChanged(LoginField, String),
    LoginFocusNext,
    LoginKeyEnter,
    LoginSubmit,
    LoginResult(Result<Option<(String, i16)>, String>),
}

// ── Update ────────────────────────────────────────────────────────────────────

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::Rest(command) => {
                self.handle_rest_command(command);
                Task::none()
            }
            Message::WindowClosed(window_id) => {
                let closed = self.windows.remove(&window_id);
                match closed {
                    Some(WindowKind::Main) => {
                        self.stop_streaming_encoder();
                        self.shutdown_rest_server();
                        self.stop_queue_players();
                        iced::exit()
                    }
                    Some(WindowKind::TrackPicker(_)) => {
                        self.stop_preview();
                        Task::none()
                    }
                    None => Task::none(),
                }
            }

            Message::PollDone => {
                let queue_log_positions = self.active_queue_play_log_positions();
                let queue_was_active = self.queue_active_flags();
                let aux_was_active = self.aux_active_flags();
                self.audio.poll();
                self.close_finished_queue_play_logs(&queue_log_positions);
                self.sync_queue_players(queue_was_active);
                self.sync_auto_mix();
                self.sync_instant_active_slot();
                self.sync_aux_loops(aux_was_active);
                if self.previewing_queue_id.is_some()
                    && !self.audio.player(PREVIEW_PLAYER_ID).is_active()
                {
                    self.previewing_queue_id = None;
                }
                Task::none()
            }

            Message::MeterTick => {
                self.update_queue_meter();
                Task::none()
            }

            Message::Stop => {
                let duration = std::time::Duration::from_millis(
                    self.app_config.stop_fade_duration_ms.max(0) as u64,
                );
                let had_active = self.any_queue_active();
                for player_id in QUEUE_PLAYER_IDS {
                    if self.audio.player(player_id).is_active() {
                        self.audio
                            .handle(player_id, audio::PlayerCommand::SoftStop(duration));
                    }
                }
                if had_active {
                    self.deck_soft_stopping = true;
                }
                self.preloaded_queue_entry = None;
                Task::none()
            }

            Message::Restart => {
                if self.audio.player(self.current_queue_player_id).is_playing() {
                    self.audio
                        .handle(self.current_queue_player_id, audio::PlayerCommand::Restart);
                }
                Task::none()
            }

            Message::TogglePlay => {
                if self.any_queue_active() {
                    for player_id in QUEUE_PLAYER_IDS {
                        if self.audio.player(player_id).is_active() {
                            self.audio
                                .handle(player_id, audio::PlayerCommand::TogglePause);
                        }
                    }
                } else {
                    self.load_next_from_queue(self.current_queue_player_id);
                }
                Task::none()
            }

            Message::Seek(offset_ms) => {
                self.audio.handle(
                    self.current_queue_player_id,
                    audio::PlayerCommand::SeekRelative(offset_ms),
                );
                Task::none()
            }

            Message::Player(id, command) => {
                self.audio.handle(id, command);
                Task::none()
            }

            Message::SearchPreviewPlay => {
                if let Some(id) = self.selected_search_track_id {
                    if let Some(path) = self.search_track_path(id) {
                        let cue_in = self.search_track_cue_in(id);
                        self.audio.handle(
                            PREVIEW_PLAYER_ID,
                            audio::PlayerCommand::Load { path, cue_in },
                        );
                        self.audio
                            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Play);
                    }
                }
                Task::none()
            }

            Message::ShowSearch => {
                self.instant_view = InstantView::Search;
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::ShowInstantPlayers => {
                self.instant_view = InstantView::InstantPlayers;
                Task::none()
            }
            Message::SearchChanged(value) => {
                self.search_query = value;
                self.search_page_start = 0;
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::CategorySelected(value) => {
                self.search_category = value;
                if self.search_subcategory.id.is_some()
                    && self.search_category.id.is_some()
                    && self.search_subcategory.parent_id != self.search_category.id
                {
                    self.search_subcategory = db::FilterOption::all(ANY_SUBCATEGORY);
                }
                self.search_page_start = 0;
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::SubcategorySelected(value) => {
                self.search_subcategory = value;
                self.search_page_start = 0;
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::GenreSelected(value) => {
                self.search_genre = value;
                self.search_page_start = 0;
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::SearchRowSelected(track_id) => {
                self.selected_search_track_id = Some(track_id);
                Task::none()
            }
            Message::SearchFirstPage => {
                self.search_page_start = 0;
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::SearchPreviousPage => {
                self.search_page_start = self.search_page_start.saturating_sub(SEARCH_PAGE_SIZE);
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::SearchNextPage => {
                let last_start = self.last_search_page_start();
                self.search_page_start =
                    (self.search_page_start + SEARCH_PAGE_SIZE).min(last_start);
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::SearchLastPage => {
                self.search_page_start = self.last_search_page_start();
                self.reload_search_tracks_from_db();
                Task::none()
            }
            Message::OpenTrackPicker(target) => {
                let (window_id, open) = window::open(picker_window_settings());
                self.windows.insert(
                    window_id,
                    WindowKind::TrackPicker(TrackPickerState::new(target)),
                );
                self.reload_picker_tracks_from_db(window_id);
                open.map(|_| Message::NoOp)
            }
            Message::PickerSearchChanged(window_id, value) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.search_query = value;
                    picker.page_start = 0;
                }
                self.reload_picker_tracks_from_db(window_id);
                Task::none()
            }
            Message::PickerCategorySelected(window_id, value) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.search_category = value;
                    if picker.search_subcategory.id.is_some()
                        && picker.search_category.id.is_some()
                        && picker.search_subcategory.parent_id != picker.search_category.id
                    {
                        picker.search_subcategory = db::FilterOption::all(ANY_SUBCATEGORY);
                    }
                    picker.page_start = 0;
                }
                self.reload_picker_tracks_from_db(window_id);
                Task::none()
            }
            Message::PickerSubcategorySelected(window_id, value) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.search_subcategory = value;
                    picker.page_start = 0;
                }
                self.reload_picker_tracks_from_db(window_id);
                Task::none()
            }
            Message::PickerGenreSelected(window_id, value) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.search_genre = value;
                    picker.page_start = 0;
                }
                self.reload_picker_tracks_from_db(window_id);
                Task::none()
            }
            Message::PickerRowPressed(window_id, track_id) => {
                let now = std::time::Instant::now();
                let mut confirmed_target = None;

                if let Some(picker) = self.track_picker_mut(window_id) {
                    let double_click = picker.last_click.is_some_and(|(last_id, clicked_at)| {
                        last_id == track_id
                            && now.duration_since(clicked_at)
                                <= std::time::Duration::from_millis(400)
                    });
                    picker.selected_track_id = Some(track_id);
                    picker.last_click = Some((track_id, now));

                    if double_click {
                        confirmed_target = Some(picker.target);
                    }
                }

                if let Some(target) = confirmed_target {
                    if let Some(track) = self.loaded_track(track_id) {
                        self.assign_loaded_track(target, track);
                        self.stop_preview();
                        return window::close(window_id);
                    }
                }

                Task::none()
            }
            Message::PickerFirstPage(window_id) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.page_start = 0;
                }
                self.reload_picker_tracks_from_db(window_id);
                Task::none()
            }
            Message::PickerPreviousPage(window_id) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.page_start = picker.page_start.saturating_sub(SEARCH_PAGE_SIZE);
                }
                self.reload_picker_tracks_from_db(window_id);
                Task::none()
            }
            Message::PickerNextPage(window_id) => {
                let last_start = self
                    .track_picker(window_id)
                    .map(|picker| self.last_picker_page_start(picker))
                    .unwrap_or(0);
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.page_start = (picker.page_start + SEARCH_PAGE_SIZE).min(last_start);
                }
                self.reload_picker_tracks_from_db(window_id);
                Task::none()
            }
            Message::PickerLastPage(window_id) => {
                let last_start = self
                    .track_picker(window_id)
                    .map(|picker| self.last_picker_page_start(picker))
                    .unwrap_or(0);
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.page_start = last_start;
                }
                self.reload_picker_tracks_from_db(window_id);
                Task::none()
            }
            Message::PickerPreviewPlay(window_id) => {
                let selected_id = self
                    .track_picker(window_id)
                    .and_then(|picker| picker.selected_track_id);
                if let Some(id) = selected_id {
                    if let Some(path) = self.search_track_path(id) {
                        let cue_in = self.search_track_cue_in(id);
                        self.audio.handle(
                            PREVIEW_PLAYER_ID,
                            audio::PlayerCommand::Load { path, cue_in },
                        );
                        self.audio
                            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Play);
                    }
                }
                Task::none()
            }
            Message::InstantSlotPressed(index) => {
                self.instant_context_slot = None;
                self.play_instant_slot(index);
                Task::none()
            }
            Message::InstantSlotRightClick(index) => {
                self.instant_context_slot = if self.instant_context_slot == Some(index) {
                    None
                } else {
                    Some(index)
                };
                Task::none()
            }
            Message::InstantSlotLoad(index) => {
                self.instant_context_slot = None;
                if let Some(slot) = self.instant_slots.get_mut(index) {
                    *slot = None;
                }
                return self.update(Message::OpenTrackPicker(PickerTarget::Instant(index)));
            }
            Message::InstantSlotClear(index) => {
                self.instant_context_slot = None;
                if self.active_instant_slot == Some(index) {
                    self.stop_instant();
                }
                if let Some(slot) = self.instant_slots.get_mut(index) {
                    *slot = None;
                }
                Task::none()
            }
            Message::InstantStop => {
                self.stop_instant();
                Task::none()
            }
            Message::InstantSave => {
                self.open_save_instant_dialog();
                Task::none()
            }
            Message::InstantSaveNameChanged(value) => {
                if let Some(Dialog::SaveInstantPage { name }) = &mut self.dialog {
                    *name = value;
                }
                Task::none()
            }
            Message::InstantSaveConfirm => {
                self.save_instant_page();
                Task::none()
            }
            Message::DialogCancel => {
                self.dialog = None;
                Task::none()
            }
            Message::AboutOpen => {
                self.dialog = Some(Dialog::About);
                Task::none()
            }
            Message::InstantNewPage => {
                self.new_instant_page();
                Task::none()
            }
            Message::InstantDeletePage => {
                self.delete_active_instant_page();
                Task::none()
            }
            Message::InstantPreviousPage => {
                self.show_previous_instant_page();
                Task::none()
            }
            Message::InstantNextPage => {
                self.show_next_instant_page();
                Task::none()
            }
            Message::ToggleAutoDj => {
                self.autodj_enabled = !self.autodj_enabled;
                if self.autodj_enabled {
                    self.set_auto_mix_status("Waiting");
                } else {
                    self.preloaded_queue_entry = None;
                    self.set_auto_mix_status("Disabled");
                }
                Task::none()
            }

            Message::QueuePreviewToggle(queue_id) => {
                if self.previewing_queue_id == Some(queue_id) {
                    self.stop_preview();
                    self.previewing_queue_id = None;
                } else {
                    let entry = self.queue_entries.iter().find(|e| e.id == queue_id);
                    let (track_id, cue_in) = entry
                        .map(|e| (e.track_id, e.cue_in))
                        .unwrap_or((None, std::time::Duration::ZERO));
                    let path = track_id.and_then(|tid| self.search_track_path(tid));
                    if let Some(path) = path {
                        self.audio.handle(
                            PREVIEW_PLAYER_ID,
                            audio::PlayerCommand::Load { path, cue_in },
                        );
                        self.audio
                            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Play);
                        self.previewing_queue_id = Some(queue_id);
                    }
                }
                Task::none()
            }

            Message::QueueMoveUp => {
                if let Some(i) = self.selected_queue_index {
                    if i > 0 {
                        self.clear_preloaded_queue_status();
                        self.queue_entries.swap(i, i - 1);
                        self.selected_queue_index = Some(i - 1);
                    }
                }
                Task::none()
            }

            Message::QueueMoveTop => {
                if let Some(i) = self.selected_queue_index {
                    if i > 0 {
                        self.clear_preloaded_queue_status();
                        let entry = self.queue_entries.remove(i);
                        self.queue_entries.insert(0, entry);
                        self.selected_queue_index = Some(0);
                    }
                }
                Task::none()
            }

            Message::QueueMoveDown => {
                if let Some(i) = self.selected_queue_index {
                    if i + 1 < self.queue_entries.len() {
                        self.clear_preloaded_queue_status();
                        self.queue_entries.swap(i, i + 1);
                        self.selected_queue_index = Some(i + 1);
                    }
                }
                Task::none()
            }

            Message::QueueMoveBottom => {
                if let Some(i) = self.selected_queue_index {
                    let last = self.queue_entries.len() - 1;
                    if i < last {
                        self.clear_preloaded_queue_status();
                        let entry = self.queue_entries.remove(i);
                        self.queue_entries.push(entry);
                        self.selected_queue_index = Some(last);
                    }
                }
                Task::none()
            }

            Message::QueueRowSelected(index) => {
                self.selected_queue_index = Some(index);
                Task::none()
            }

            Message::QueuePlayNow(index) => {
                self.play_queue_entry_now(index);
                Task::none()
            }

            Message::QueueInsertTrack => {
                self.insert_selected_search_into_queue();
                Task::none()
            }

            Message::QueueReplaceTrack => {
                self.replace_selected_queue_entry();
                Task::none()
            }

            Message::QueueRemoveEntry => {
                self.remove_selected_queue_entry();
                Task::none()
            }

            Message::QueueClearAll => {
                self.queue_entries.clear();
                self.selected_queue_index = None;
                self.clear_preloaded_queue_status();
                Task::none()
            }

            Message::QueueReload => {
                self.reload_queue_entries_from_db();
                Task::none()
            }

            Message::AuxPlay(index) => {
                self.play_aux_slot(index);
                Task::none()
            }
            Message::AuxStop(index) => {
                self.stop_aux_slot(index);
                Task::none()
            }
            Message::AuxToggleLoop(index) => {
                self.toggle_aux_loop(index);
                Task::none()
            }
            Message::ClockTick => {
                self.current_hour = current_hour();
                self.current_date = current_date();
                Task::none()
            }

            Message::DbConfigOpen => {
                let (host, port, database, user, password, psql_path) =
                    if let Ok(raw) = std::fs::read_to_string(db_config_path()) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                            (
                                val["host"].as_str().unwrap_or("localhost").to_string(),
                                val["port"].as_u64().unwrap_or(5432).to_string(),
                                val["database"].as_str().unwrap_or("openstudio").to_string(),
                                val["user"].as_str().unwrap_or("postgres").to_string(),
                                val["password"].as_str().unwrap_or("").to_string(),
                                val["psql_path"]
                                    .as_str()
                                    .unwrap_or(DEFAULT_PSQL_PATH)
                                    .to_string(),
                            )
                        } else {
                            (
                                "localhost".into(),
                                "5432".into(),
                                "openstudio".into(),
                                "postgres".into(),
                                String::new(),
                                DEFAULT_PSQL_PATH.into(),
                            )
                        }
                    } else {
                        (
                            "localhost".into(),
                            "5432".into(),
                            "openstudio".into(),
                            "postgres".into(),
                            String::new(),
                            DEFAULT_PSQL_PATH.into(),
                        )
                    };
                self.dialog = Some(Dialog::EditDbConfig {
                    host,
                    port,
                    database,
                    user,
                    password,
                    psql_path,
                    connection_status: None,
                    create_status: None,
                    delete_status: None,
                    delete_confirm: false,
                });
                Task::none()
            }

            Message::DbConfigFieldChanged(field, value) => {
                if let Some(Dialog::EditDbConfig {
                    host,
                    port,
                    database,
                    user,
                    password,
                    psql_path,
                    ..
                }) = &mut self.dialog
                {
                    match field {
                        DbField::Host => *host = value,
                        DbField::Port => *port = value,
                        DbField::Database => *database = value,
                        DbField::User => *user = value,
                        DbField::Password => *password = value,
                        DbField::PsqlPath => *psql_path = value,
                    }
                }
                Task::none()
            }

            Message::DbConfigSave => {
                if let Some(Dialog::EditDbConfig {
                    host,
                    port,
                    database,
                    user,
                    password,
                    psql_path,
                    ..
                }) = &self.dialog
                {
                    let config = serde_json::json!({
                        "host": host,
                        "port": port.parse::<u16>().unwrap_or(5432),
                        "database": database,
                        "user": user,
                        "password": password,
                        "psql_path": psql_path,
                    });
                    let path = db_config_save_path();
                    match serde_json::to_string_pretty(&config)
                        .map_err(|e| e.to_string())
                        .and_then(|json| {
                            if let Some(parent) = path.parent() {
                                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                            }
                            std::fs::write(&path, json).map_err(|e| e.to_string())
                        }) {
                        Err(e) => {
                            self.status = format!("Config write failed: {e}");
                            return Task::none();
                        }
                        Ok(()) => {}
                    }
                    self.dialog = None;
                    self.db = None;
                    self.status = "Reconnecting...".into();
                    Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                db::Database::connect_from_file(&path).map_err(|e| e.to_string())
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::DbConfigConnected,
                    )
                } else {
                    Task::none()
                }
            }

            Message::DbConfigConnected(result) => {
                match result {
                    Ok(db) => {
                        self.status = "Connected".into();
                        match db.timezones() {
                            Ok(options) => self.timezone_options = options,
                            Err(error) => {
                                self.status = format!("Connected (partial: timezones: {error})")
                            }
                        }
                        self.db = Some(db);
                        self.ensure_configured_timezone_option();
                        let auto_mix_status = self.auto_mix_status.clone();
                        self.log_auto_mix_status(&auto_mix_status);
                    }
                    Err(e) => {
                        self.db = None;
                        self.status = format!("Disconnected ({e})");
                    }
                }
                Task::none()
            }

            Message::DbConfigTestConnection => {
                if let Some(Dialog::EditDbConfig {
                    host,
                    port,
                    user,
                    password,
                    psql_path,
                    connection_status,
                    ..
                }) = &mut self.dialog
                {
                    *connection_status = Some(Ok("Testing...".into()));
                    let host = host.clone();
                    let port = port.clone();
                    let user = user.clone();
                    let password = password.clone();
                    let psql_path = psql_path.clone();
                    Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                let out = std::process::Command::new(&psql_path)
                                    .env("PGPASSWORD", &password)
                                    .args([
                                        "-U",
                                        &user,
                                        "-h",
                                        &host,
                                        "-p",
                                        &port,
                                        "-c",
                                        "SELECT version();",
                                        "postgres",
                                    ])
                                    .output();
                                match out {
                                    Ok(o) if o.status.success() => {
                                        let v = String::from_utf8_lossy(&o.stdout);
                                        let version =
                                            v.lines().nth(2).unwrap_or("").trim().to_string();
                                        Ok(if version.is_empty() {
                                            "Connected".into()
                                        } else {
                                            version
                                        })
                                    }
                                    Ok(o) => {
                                        Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
                                    }
                                    Err(e) => Err(e.to_string()),
                                }
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::DbConfigTestResult,
                    )
                } else {
                    Task::none()
                }
            }

            Message::DbConfigTestResult(result) => {
                if let Some(Dialog::EditDbConfig {
                    connection_status, ..
                }) = &mut self.dialog
                {
                    *connection_status = Some(result);
                }
                Task::none()
            }

            Message::DbConfigCreateDatabase => {
                if let Some(Dialog::EditDbConfig {
                    host,
                    port,
                    database,
                    user,
                    password,
                    psql_path,
                    create_status,
                    ..
                }) = &mut self.dialog
                {
                    *create_status = Some(Ok("Creating...".into()));
                    let host = host.clone();
                    let port = port.clone();
                    let database = database.clone();
                    let user = user.clone();
                    let password = password.clone();
                    let psql_path = psql_path.clone();
                    let mig_dir = migrations_dir();
                    Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                let run = |args: &[&str]| -> Result<(), String> {
                                    let out = std::process::Command::new(&psql_path)
                                        .env("PGPASSWORD", &password)
                                        .args(args)
                                        .output()
                                        .map_err(|e| e.to_string())?;
                                    if out.status.success() {
                                        Ok(())
                                    } else {
                                        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
                                    }
                                };

                                run(&[
                                    "-U",
                                    &user,
                                    "-h",
                                    &host,
                                    "-p",
                                    &port,
                                    "-c",
                                    &format!("CREATE DATABASE \"{database}\""),
                                    "postgres",
                                ])?;

                                let m1 = mig_dir.join("0001_initial_schema.sql");
                                let m2 = mig_dir.join("0002_seed.sql");
                                run(&[
                                    "-U",
                                    &user,
                                    "-h",
                                    &host,
                                    "-p",
                                    &port,
                                    "-f",
                                    m1.to_str().unwrap_or(""),
                                    &database,
                                ])?;
                                run(&[
                                    "-U",
                                    &user,
                                    "-h",
                                    &host,
                                    "-p",
                                    &port,
                                    "-f",
                                    m2.to_str().unwrap_or(""),
                                    &database,
                                ])?;

                                Ok(format!("Database \"{database}\" created and initialized."))
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::DbConfigCreateResult,
                    )
                } else {
                    Task::none()
                }
            }

            Message::DbConfigCreateResult(result) => {
                if let Some(Dialog::EditDbConfig { create_status, .. }) = &mut self.dialog {
                    *create_status = Some(result);
                }
                Task::none()
            }

            Message::DbConfigAskDeleteDatabase => {
                if let Some(Dialog::EditDbConfig {
                    delete_confirm,
                    delete_status,
                    ..
                }) = &mut self.dialog
                {
                    *delete_confirm = true;
                    *delete_status = None;
                }
                Task::none()
            }

            Message::DbConfigCancelDeleteDatabase => {
                if let Some(Dialog::EditDbConfig { delete_confirm, .. }) = &mut self.dialog {
                    *delete_confirm = false;
                }
                Task::none()
            }

            Message::DbConfigDeleteDatabase => {
                if let Some(Dialog::EditDbConfig {
                    host,
                    port,
                    database,
                    user,
                    password,
                    psql_path,
                    delete_status,
                    ..
                }) = &mut self.dialog
                {
                    let database = database.trim().to_string();
                    if database.is_empty() {
                        *delete_status = Some(Err("Database name is required.".into()));
                        return Task::none();
                    }
                    if matches!(database.as_str(), "postgres" | "template0" | "template1") {
                        *delete_status = Some(Err(format!(
                            "Refusing to drop protected PostgreSQL database \"{database}\"."
                        )));
                        return Task::none();
                    }

                    *delete_status = Some(Ok("Dropping database...".into()));
                    let host = host.clone();
                    let port = port.clone();
                    let user = user.clone();
                    let password = password.clone();
                    let psql_path = psql_path.clone();
                    Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                let sql = format!(
                                    "DROP DATABASE IF EXISTS {} WITH (FORCE)",
                                    pg_quote_ident(&database)
                                );
                                let out = std::process::Command::new(&psql_path)
                                    .env("PGPASSWORD", &password)
                                    .args([
                                        "-U", &user, "-h", &host, "-p", &port, "-c", &sql,
                                        "postgres",
                                    ])
                                    .output()
                                    .map_err(|e| e.to_string())?;
                                if out.status.success() {
                                    Ok(format!("Database \"{database}\" was permanently deleted."))
                                } else {
                                    Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
                                }
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::DbConfigDeleteResult,
                    )
                } else {
                    Task::none()
                }
            }

            Message::DbConfigDeleteResult(result) => {
                if let Some(Dialog::EditDbConfig {
                    delete_status,
                    delete_confirm,
                    ..
                }) = &mut self.dialog
                {
                    if result.is_ok() {
                        self.db = None;
                        self.status = "Disconnected (database deleted)".into();
                        *delete_confirm = false;
                    }
                    *delete_status = Some(result);
                }
                Task::none()
            }

            Message::ConfigOpen => {
                if !matches!(self.current_user_role, 1 | 2) {
                    self.login_pending = Some(PendingLogin::ConfigOpen);
                    self.dialog = Some(Dialog::Login {
                        login: String::new(),
                        password: String::new(),
                        error: None,
                        focus_index: 0,
                    });
                    return iced::widget::text_input::focus(login_input_id());
                }
                self.audio_devices = audio::list_output_devices();
                self.audio_input_devices = audio::list_input_devices();
                let device_deck_id = audio::normalize_device_id(
                    &self.app_config.device_deck_id,
                    &self.audio_devices,
                );
                let device_instant_id = audio::normalize_device_id(
                    &self.app_config.device_instant_id,
                    &self.audio_devices,
                );
                let device_aux_id =
                    audio::normalize_device_id(&self.app_config.device_aux_id, &self.audio_devices);
                let device_preview_id = audio::normalize_device_id(
                    &self.app_config.device_preview_id,
                    &self.audio_devices,
                );
                let encoder_input_device_id = audio::normalize_device_id(
                    &self.app_config.encoder_input_device_id,
                    &self.audio_input_devices,
                );
                self.dialog = Some(Dialog::EditConfig {
                    auto_mix_on_start: self.app_config.auto_mix_on_start,
                    auto_play_on_start: self.app_config.auto_play_on_start,
                    start_locked: self.app_config.start_locked,
                    preload: self.app_config.preload.to_string(),
                    fade_out_duration_ms: self.app_config.fade_out_duration_ms.to_string(),
                    stop_fade_duration_ms: self.app_config.stop_fade_duration_ms.to_string(),
                    timezone: self.app_config.timezone.clone(),
                    device_deck_id,
                    device_instant_id,
                    device_aux_id,
                    device_preview_id,
                    encoder_input_device_id,
                });
                Task::none()
            }

            Message::AudioProcessingOpen => {
                self.dialog = Some(Dialog::AudioProcessing {
                    processing_bypassed: self.audio.processing_bypassed(),
                    input_volume: self.audio.master_volume_percent(),
                    compressor_mode: self.app_config.audio_compressor_mode.clone(),
                    compressor_preset: self.app_config.audio_compressor_preset.clone(),
                    attack: format!("{:.1}", self.audio.compressor_attack_ms()),
                    ratio: format!("{:.2}", self.audio.compressor_ratio()),
                    threshold: format!("{:.1}", self.audio.compressor_threshold_db()),
                    gain: format!("{:.1}", self.audio.compressor_gain_db()),
                    release: format!("{:.1}", self.audio.compressor_release_ms()),
                    eq_enabled: self.audio.eq_enabled(),
                    eq_gains: self.audio.eq_gains_db(),
                    agc_preset: self.audio.agc_preset().as_str().into(),
                });
                Task::none()
            }

            Message::AudioProcessingBypassChanged(value) => {
                self.audio.set_processing_bypassed(value);
                if let Some(Dialog::AudioProcessing {
                    processing_bypassed,
                    ..
                }) = &mut self.dialog
                {
                    *processing_bypassed = value;
                }
                Task::none()
            }

            Message::AudioProcessingInputVolumeChanged(value) => {
                let value = value.clamp(0.0, 100.0);
                self.audio.set_master_volume_percent(value);
                if let Some(Dialog::AudioProcessing { input_volume, .. }) = &mut self.dialog {
                    *input_volume = value;
                }
                Task::none()
            }

            Message::AudioProcessingModeChanged(value) => {
                let should_apply_preset = value == "By Preset";
                let mut preset_to_apply = None;
                if let Some(Dialog::AudioProcessing {
                    compressor_mode,
                    compressor_preset,
                    ..
                }) = &mut self.dialog
                {
                    *compressor_mode = value;
                    if should_apply_preset {
                        preset_to_apply = find_compressor_preset(compressor_preset);
                    }
                }
                if let Some(preset) = preset_to_apply {
                    self.apply_compressor_preset(preset);
                }
                Task::none()
            }

            Message::AudioProcessingPresetChanged(value) => {
                let preset_to_apply = find_compressor_preset(&value);
                if let Some(Dialog::AudioProcessing {
                    compressor_mode,
                    compressor_preset,
                    ..
                }) = &mut self.dialog
                {
                    *compressor_mode = "By Preset".into();
                    *compressor_preset = value;
                }
                if let Some(preset) = preset_to_apply {
                    self.apply_compressor_preset(preset);
                }
                Task::none()
            }

            Message::AudioProcessingAttackChanged(value) => {
                if let Ok(parsed) = value.parse::<f32>() {
                    self.audio.set_compressor_attack_ms(parsed);
                }
                if let Some(Dialog::AudioProcessing {
                    compressor_mode,
                    attack,
                    ..
                }) = &mut self.dialog
                {
                    *compressor_mode = "Custom Values".into();
                    *attack = value;
                }
                Task::none()
            }

            Message::AudioProcessingRatioChanged(value) => {
                if let Ok(parsed) = value.parse::<f32>() {
                    self.audio.set_compressor_ratio(parsed);
                }
                if let Some(Dialog::AudioProcessing {
                    compressor_mode,
                    ratio,
                    ..
                }) = &mut self.dialog
                {
                    *compressor_mode = "Custom Values".into();
                    *ratio = value;
                }
                Task::none()
            }

            Message::AudioProcessingThresholdChanged(value) => {
                if let Ok(parsed) = value.parse::<f32>() {
                    self.audio.set_compressor_threshold_db(parsed);
                }
                if let Some(Dialog::AudioProcessing {
                    compressor_mode,
                    threshold,
                    ..
                }) = &mut self.dialog
                {
                    *compressor_mode = "Custom Values".into();
                    *threshold = value;
                }
                Task::none()
            }

            Message::AudioProcessingGainChanged(value) => {
                if let Ok(parsed) = value.parse::<f32>() {
                    self.audio.set_compressor_gain_db(parsed);
                }
                if let Some(Dialog::AudioProcessing {
                    compressor_mode,
                    gain,
                    ..
                }) = &mut self.dialog
                {
                    *compressor_mode = "Custom Values".into();
                    *gain = value;
                }
                Task::none()
            }

            Message::AudioProcessingReleaseChanged(value) => {
                if let Ok(parsed) = value.parse::<f32>() {
                    self.audio.set_compressor_release_ms(parsed);
                }
                if let Some(Dialog::AudioProcessing {
                    compressor_mode,
                    release,
                    ..
                }) = &mut self.dialog
                {
                    *compressor_mode = "Custom Values".into();
                    *release = value;
                }
                Task::none()
            }

            Message::AudioProcessingEqEnabledChanged(value) => {
                self.audio.set_eq_enabled(value);
                if let Some(Dialog::AudioProcessing { eq_enabled, .. }) = &mut self.dialog {
                    *eq_enabled = value;
                }
                Task::none()
            }

            Message::AudioProcessingEqGainChanged(index, value) => {
                self.audio.set_eq_gain_db(index, value);
                if let Some(Dialog::AudioProcessing { eq_gains, .. }) = &mut self.dialog {
                    if let Some(gain) = eq_gains.get_mut(index) {
                        *gain = value;
                    }
                }
                Task::none()
            }

            Message::AudioProcessingAgcPresetChanged(value) => {
                if let Some(preset) = audio::AgcPreset::from_str(&value) {
                    self.audio.set_agc_preset(preset);
                }
                if let Some(Dialog::AudioProcessing { agc_preset, .. }) = &mut self.dialog {
                    *agc_preset = value;
                }
                Task::none()
            }

            Message::AudioProcessingSave => {
                let Some(cfg) = self.audio_processing_config_from_dialog() else {
                    return Task::none();
                };
                self.apply_audio_processing_config(&cfg);
                self.app_config = cfg.clone();
                self.dialog = None;
                if let Some(db) = self.db.clone() {
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                db.save_config(&cfg).map_err(|e| e.to_string())
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::ConfigSaved,
                    );
                }
                Task::none()
            }

            Message::StreamEncoderOpen => {
                self.open_stream_encoder_dialog();
                Task::none()
            }

            Message::StreamEncoderEnabledChanged(value) => {
                self.update_stream_encoder_dialog(|dialog| {
                    if let Dialog::StreamEncoder { enabled, error, .. } = dialog {
                        *enabled = value;
                        *error = None;
                    }
                })
            }

            Message::StreamEncoderBitrateChanged(value) => {
                self.set_stream_encoder_bitrate_input(value)
            }

            Message::StreamEncoderSampleRateChanged(value) => {
                self.set_stream_encoder_sample_rate_input(value)
            }

            Message::StreamEncoderTypeChanged(value) => self.set_stream_encoder_type_input(value),

            Message::StreamEncoderChannelsChanged(value) => {
                self.set_stream_encoder_channels_input(value)
            }

            Message::StreamEncoderPasswordChanged(value) => {
                self.update_stream_encoder_dialog(|dialog| {
                    if let Dialog::StreamEncoder {
                        password, error, ..
                    } = dialog
                    {
                        *password = value;
                        *error = None;
                    }
                })
            }

            Message::StreamEncoderMountpointChanged(value) => {
                self.set_stream_encoder_mountpoint_input(value)
            }

            Message::StreamEncoderReconnectChanged(value) => {
                self.set_stream_encoder_reconnect_input(value)
            }

            Message::StreamEncoderSave => {
                if self.stream_encoder_enabled_in_dialog()
                    && !self.stream_encoder_password_is_valid()
                {
                    self.set_stream_encoder_error("Encoder password is required.");
                    return Task::none();
                }
                let Some(cfg) = self.stream_encoder_config_from_dialog() else {
                    return Task::none();
                };
                self.app_config = cfg.clone();
                self.sync_streaming_encoder();
                self.dialog = None;
                if let Some(db) = self.db.clone() {
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                db.save_config(&cfg).map_err(|e| e.to_string())
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::ConfigSaved,
                    );
                }
                Task::none()
            }

            Message::ConfigToggle(field) => {
                if let Some(Dialog::EditConfig {
                    auto_mix_on_start,
                    auto_play_on_start,
                    start_locked,
                    ..
                }) = &mut self.dialog
                {
                    match field {
                        ConfigField::AutoMixOnStart => *auto_mix_on_start = !*auto_mix_on_start,
                        ConfigField::AutoPlayOnStart => *auto_play_on_start = !*auto_play_on_start,
                        ConfigField::StartLocked => *start_locked = !*start_locked,
                    }
                }
                Task::none()
            }

            Message::ConfigPreloadChanged(value) => {
                if let Some(Dialog::EditConfig { preload, .. }) = &mut self.dialog {
                    *preload = value;
                }
                Task::none()
            }

            Message::ConfigFadeOutDurationChanged(value) => {
                if let Some(Dialog::EditConfig {
                    fade_out_duration_ms,
                    ..
                }) = &mut self.dialog
                {
                    *fade_out_duration_ms = value;
                }
                Task::none()
            }

            Message::ConfigStopFadeDurationChanged(value) => {
                if let Some(Dialog::EditConfig {
                    stop_fade_duration_ms,
                    ..
                }) = &mut self.dialog
                {
                    *stop_fade_duration_ms = value;
                }
                Task::none()
            }

            Message::ConfigTimezoneChanged(value) => {
                if let Some(Dialog::EditConfig { timezone, .. }) = &mut self.dialog {
                    *timezone = value;
                }
                Task::none()
            }

            Message::ConfigDeviceChanged(target, name) => {
                if let Some(Dialog::EditConfig {
                    device_deck_id,
                    device_instant_id,
                    device_aux_id,
                    device_preview_id,
                    encoder_input_device_id,
                    ..
                }) = &mut self.dialog
                {
                    match target {
                        DeviceTarget::Deck => *device_deck_id = name,
                        DeviceTarget::Instant => *device_instant_id = name,
                        DeviceTarget::Aux => *device_aux_id = name,
                        DeviceTarget::Preview => *device_preview_id = name,
                        DeviceTarget::StreamInput => *encoder_input_device_id = name,
                    }
                }
                Task::none()
            }

            Message::ConfigSave => {
                if let Some(Dialog::EditConfig {
                    auto_mix_on_start,
                    auto_play_on_start,
                    start_locked,
                    preload,
                    fade_out_duration_ms,
                    stop_fade_duration_ms,
                    timezone,
                    device_deck_id,
                    device_instant_id,
                    device_aux_id,
                    device_preview_id,
                    encoder_input_device_id,
                }) = &self.dialog
                {
                    let mut cfg = self.app_config.clone();
                    cfg.auto_mix_on_start = *auto_mix_on_start;
                    cfg.auto_play_on_start = *auto_play_on_start;
                    cfg.start_locked = *start_locked;
                    cfg.preload = preload.trim().parse::<i32>().unwrap_or(10).max(0);
                    cfg.fade_out_duration_ms = fade_out_duration_ms
                        .trim()
                        .parse::<i32>()
                        .unwrap_or(2500)
                        .max(0);
                    cfg.stop_fade_duration_ms = stop_fade_duration_ms
                        .trim()
                        .parse::<i32>()
                        .unwrap_or(1000)
                        .max(0);
                    cfg.timezone = timezone.clone();
                    cfg.device_deck_id = device_deck_id.clone();
                    cfg.device_instant_id = device_instant_id.clone();
                    cfg.device_aux_id = device_aux_id.clone();
                    cfg.device_preview_id = device_preview_id.clone();
                    cfg.encoder_input_device_id = encoder_input_device_id.clone();
                    self.apply_audio_device_config(&cfg);
                    self.app_config = cfg.clone();
                    self.sync_streaming_encoder();
                    self.ensure_configured_timezone_option();
                    self.reload_queue_entries_from_db();
                    self.dialog = None;
                    if let Some(db) = self.db.clone() {
                        return Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    db.save_config(&cfg).map_err(|e| e.to_string())
                                })
                                .await
                                .unwrap_or_else(|e| Err(e.to_string()))
                            },
                            Message::ConfigSaved,
                        );
                    }
                }
                Task::none()
            }

            Message::ConfigSaved(result) => {
                if let Err(e) = result {
                    self.status = format!("Config save failed: {e}");
                }
                Task::none()
            }

            Message::CloseRequested(window_id) => {
                if Some(window_id) == self.main_window {
                    self.dialog = Some(Dialog::ConfirmClose { window_id });
                } else {
                    return window::close(window_id);
                }
                Task::none()
            }

            Message::ConfirmQuit => {
                if let Some(Dialog::ConfirmClose { window_id }) = self.dialog {
                    self.stop_streaming_encoder();
                    self.shutdown_rest_server();
                    return window::close(window_id);
                }
                Task::none()
            }

            Message::LockToggle => {
                if self.is_locked {
                    self.dialog = Some(Dialog::Login {
                        login: String::new(),
                        password: String::new(),
                        error: None,
                        focus_index: 0,
                    });
                    return iced::widget::text_input::focus(login_input_id());
                } else {
                    self.current_user_login = String::from("user");
                    self.current_user_role = 0;
                    self.login_pending = None;
                    self.is_locked = true;
                }
                Task::none()
            }

            Message::LoginFieldChanged(field, value) => {
                if let Some(Dialog::Login {
                    login, password, ..
                }) = &mut self.dialog
                {
                    match field {
                        LoginField::Login => *login = value,
                        LoginField::Password => *password = value,
                    }
                }
                Task::none()
            }

            Message::LoginFocusNext => {
                if let Some(Dialog::Login { focus_index, .. }) = &mut self.dialog {
                    *focus_index = (*focus_index + 1) % 4;
                    let idx = *focus_index;
                    return match idx {
                        0 => iced::widget::text_input::focus(login_input_id()),
                        1 => iced::widget::text_input::focus(pass_input_id()),
                        // Blur tous les text_inputs pour que Enter atteigne la subscription
                        _ => iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                            "__none__",
                        )),
                    };
                }
                Task::none()
            }

            Message::LoginKeyEnter => {
                if let Some(Dialog::Login { focus_index, .. }) = &self.dialog {
                    if *focus_index == 3 {
                        self.dialog = None;
                        return Task::none();
                    }
                }
                self.update(Message::LoginSubmit)
            }

            Message::LoginSubmit => {
                if let Some(Dialog::Login {
                    login, password, ..
                }) = &self.dialog
                {
                    if let Some(db) = self.db.clone() {
                        let login = login.clone();
                        let password = password.clone();
                        return Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    db.check_credentials(&login, &password)
                                        .map_err(|e| e.to_string())
                                })
                                .await
                                .unwrap_or_else(|e| Err(e.to_string()))
                            },
                            Message::LoginResult,
                        );
                    }
                }
                Task::none()
            }

            Message::LoginResult(result) => {
                match result {
                    Ok(Some((login, role))) => {
                        self.current_user_login = login;
                        self.current_user_role = role;
                        self.is_locked = false;
                        self.dialog = None;
                        if let Some(pending) = self.login_pending.take() {
                            return self.update(match pending {
                                PendingLogin::ConfigOpen => Message::ConfigOpen,
                            });
                        }
                    }
                    Ok(None) => {
                        if let Some(Dialog::Login { error, .. }) = &mut self.dialog {
                            *error = Some("Login ou mot de passe incorrect.".into());
                        }
                    }
                    Err(e) => {
                        if let Some(Dialog::Login { error, .. }) = &mut self.dialog {
                            *error = Some(format!("Erreur: {e}"));
                        }
                    }
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let keyboard = if matches!(&self.dialog, Some(Dialog::Login { .. })) {
            iced::keyboard::on_key_press(|key, _| match key {
                Key::Named(Named::Tab) => Some(Message::LoginFocusNext),
                Key::Named(Named::Enter) => Some(Message::LoginKeyEnter),
                _ => None,
            })
        } else if !self.is_locked {
            iced::keyboard::on_key_press(|key, _modifiers| {
                matches!(key, Key::Named(Named::Space)).then_some(Message::TogglePlay)
            })
        } else {
            Subscription::none()
        };
        let clock =
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::ClockTick);
        let windows = window::close_events().map(Message::WindowClosed);
        let close_requests = window::close_requests().map(Message::CloseRequested);
        let mut subscriptions = vec![keyboard, clock, windows, close_requests];

        if self.audio.any_active() {
            let poll =
                iced::time::every(std::time::Duration::from_millis(250)).map(|_| Message::PollDone);
            subscriptions.push(poll);
        }

        if self.queue_meter_needs_tick() {
            let meter = iced::time::every(std::time::Duration::from_millis(METER_TICK_MS))
                .map(|_| Message::MeterTick);
            subscriptions.push(meter);
        }

        subscriptions.push(self.rest_command_subscription());

        Subscription::batch(subscriptions)
    }

    fn rest_command_subscription(&self) -> Subscription<Message> {
        let receiver = Arc::clone(&self.rest_rx);

        Subscription::run_with_id(
            "openstudio-rest-api",
            iced::stream::channel(100, move |mut output| async move {
                use iced::futures::SinkExt;

                let rx = {
                    let Ok(mut guard) = receiver.lock() else {
                        return;
                    };
                    guard.take()
                };

                let Some(rx) = rx else {
                    return;
                };
                let rx = Arc::new(Mutex::new(rx));

                loop {
                    let command = tokio::task::spawn_blocking({
                        let rx = Arc::clone(&rx);
                        move || {
                            let Ok(guard) = rx.lock() else {
                                return None;
                            };
                            guard.recv().ok()
                        }
                    })
                    .await;

                    match command {
                        Ok(Some(command)) => {
                            if output.send(Message::Rest(command)).await.is_err() {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
            }),
        )
    }

    fn shutdown_rest_server(&mut self) {
        if let Some(tx) = self.rest_shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    fn title(&self, window_id: window::Id) -> String {
        match self.windows.get(&window_id) {
            Some(WindowKind::TrackPicker(picker)) => match picker.target {
                PickerTarget::Instant(index) => format!("Load Instant {}", index + 1),
                PickerTarget::Aux(index) => format!("Load AUX {}", index + 1),
            },
            _ => String::from("OpenStudio"),
        }
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::Dark
    }

    fn track_picker(&self, window_id: window::Id) -> Option<&TrackPickerState> {
        match self.windows.get(&window_id) {
            Some(WindowKind::TrackPicker(picker)) => Some(picker),
            _ => None,
        }
    }

    fn track_picker_mut(&mut self, window_id: window::Id) -> Option<&mut TrackPickerState> {
        match self.windows.get_mut(&window_id) {
            Some(WindowKind::TrackPicker(picker)) => Some(picker),
            _ => None,
        }
    }

    fn transport(&self) -> audio::PlayerSnapshot {
        self.audio.player(self.current_queue_player_id).snapshot()
    }

    fn elapsed(&self) -> std::time::Duration {
        self.transport().position
    }

    fn transport_duration(&self) -> Option<std::time::Duration> {
        self.transport().duration
    }

    fn ui_playing(&self) -> bool {
        QUEUE_PLAYER_IDS
            .iter()
            .any(|&player_id| self.audio.player(player_id).is_playing())
    }

    pub(crate) fn queue_meter_levels(&self) -> (f32, f32) {
        (self.queue_meter_left, self.queue_meter_right)
    }

    fn queue_meter_needs_tick(&self) -> bool {
        self.any_queue_active() || self.queue_meter_left > 0.0 || self.queue_meter_right > 0.0
    }

    fn update_queue_meter(&mut self) {
        let (left, right) = QUEUE_PLAYER_IDS
            .iter()
            .filter(|&&player_id| self.audio.player(player_id).is_active())
            .map(|&player_id| self.audio.player(player_id).levels())
            .fold(
                (0.0_f32, 0.0_f32),
                |(left_max, right_max), (left, right)| (left_max.max(left), right_max.max(right)),
            );

        self.queue_meter_left = smooth_meter(self.queue_meter_left, audio_peak_to_meter(left));
        self.queue_meter_right = smooth_meter(self.queue_meter_right, audio_peak_to_meter(right));
    }

    fn track_title(&self) -> String {
        let entry = self
            .current_queue_entry
            .as_ref()
            .or_else(|| self.queue_entries.first());
        match entry {
            None => String::from("—"),
            Some(e) => match e.title.trim() {
                "" => format!("Queue item {}", e.id),
                title => title.to_string(),
            },
        }
    }

    fn track_artist(&self) -> String {
        let entry = self
            .current_queue_entry
            .as_ref()
            .or_else(|| self.queue_entries.first());
        match entry {
            None => String::from("—"),
            Some(e) => match e.artist_name.trim() {
                "" => String::from("—"),
                artist => artist.to_string(),
            },
        }
    }

    fn file_name(&self) -> String {
        let entry = self
            .current_queue_entry
            .as_ref()
            .or_else(|| self.queue_entries.first());
        match entry {
            None => String::from("Queue vide"),
            Some(e) => {
                let scheduled = e.scheduled_at.as_deref().unwrap_or("—");
                format!("Scheduled {scheduled}")
            }
        }
    }

    fn db_status_display(&self) -> String {
        if self.db.is_some() {
            if self.status.starts_with("Connected") {
                self.status.clone()
            } else {
                String::from("Connected")
            }
        } else if self.status.starts_with("Disconnected") || self.status.starts_with("Reconnecting")
        {
            self.status.clone()
        } else {
            String::from("Disconnected")
        }
    }

    fn ensure_configured_timezone_option(&mut self) {
        if !self.timezone_options.contains(&self.app_config.timezone) {
            self.timezone_options.push(self.app_config.timezone.clone());
            self.timezone_options.sort();
        }
    }
}

// ── View ──────────────────────────────────────────────────────────────────────
