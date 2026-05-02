mod audio;
mod db;
mod ui;

use std::collections::HashMap;
use std::path::PathBuf;

use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::widget::{
    button, checkbox, column, container, responsive, row, stack, text, text_input, Space,
};
use iced::{
    window, Alignment, Background, Border, Color, Element, Length, Size, Subscription, Task, Theme,
};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};
use ui::{accent_purple, block_style, panel_style, rgb, text_color};

fn db_config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| {
            let candidate = exe.parent()?.parent()?.join("Resources/database.json");
            candidate.exists().then_some(candidate)
        })
        .unwrap_or_else(|| PathBuf::from("config/database.json"))
}

const ANY_CATEGORY: &str = "Any Category";
const ANY_SUBCATEGORY: &str = "Any Subcategory";
const ANY_GENRE: &str = "Any Genre";
const SEARCH_PAGE_SIZE: usize = 50;
const QUEUE_PLAYER_IDS: [audio::PlayerId; 2] = [audio::PlayerId::QueueA, audio::PlayerId::QueueB];
const PREVIEW_PLAYER_ID: audio::PlayerId = audio::PlayerId::Preview;
const INSTANT_PLAYER_ID: audio::PlayerId = audio::PlayerId::Instant;
const AUX_PLAYER_IDS: [audio::PlayerId; 3] = [
    audio::PlayerId::Aux1,
    audio::PlayerId::Aux2,
    audio::PlayerId::Aux3,
];
const METER_TICK_MS: u64 = 100;
const METER_DECAY_PER_SECOND: f32 = 0.32;

fn main() -> iced::Result {
    iced::daemon(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .run_with(App::new)
}

fn main_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(1240.0, 820.0),
        min_size: Some(Size::new(960.0, 640.0)),
        position: window::Position::Centered,
        ..window::Settings::default()
    }
}

fn picker_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(980.0, 680.0),
        min_size: Some(Size::new(780.0, 520.0)),
        position: window::Position::Centered,
        ..window::Settings::default()
    }
}

fn audio_peak_to_meter(peak: f32) -> f32 {
    if peak <= 0.000_001 {
        return 0.0;
    }

    let db = 20.0 * peak.clamp(0.000_001, 1.0).log10();
    ((db + 60.0) / 60.0).clamp(0.0, 1.0)
}

fn smooth_meter(current: f32, target: f32) -> f32 {
    if target >= current {
        target
    } else {
        let decay = METER_DECAY_PER_SECOND * METER_TICK_MS as f32 / 1000.0;
        (current - decay).max(target).max(0.0)
    }
}

fn auto_mix_trigger(entry: &db::QueueEntry) -> std::time::Duration {
    if entry.cue_out > std::time::Duration::ZERO && entry.cue_out < entry.duration {
        entry.cue_out
    } else {
        entry.duration
    }
}

// ── Time helpers ─────────────────────────────────────────────────────────────

#[cfg(unix)]
fn system_locale_cstr() -> &'static std::ffi::CString {
    static LOCALE: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();
    LOCALE.get_or_init(|| {
        #[cfg(target_os = "macos")]
        if let Ok(out) = std::process::Command::new("defaults")
            .args(["read", "NSGlobalDomain", "AppleLocale"])
            .output()
        {
            if let Ok(s) = std::str::from_utf8(&out.stdout) {
                let s = s.trim();
                if !s.is_empty() {
                    let locale = if s.contains('.') {
                        s.to_string()
                    } else {
                        format!("{}.UTF-8", s)
                    };
                    if let Ok(cs) = std::ffi::CString::new(locale) {
                        return cs;
                    }
                }
            }
        }
        std::ffi::CString::new("").unwrap()
    })
}

#[cfg(unix)]
fn current_hour() -> String {
    let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return String::from("--:--:--");
    };
    let timestamp = now.as_secs() as libc::time_t;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
    let local = unsafe {
        if libc::localtime_r(&timestamp, local.as_mut_ptr()).is_null() {
            return String::from("--:--:--");
        }
        local.assume_init()
    };
    format!(
        "{:02}:{:02}:{:02}",
        local.tm_hour, local.tm_min, local.tm_sec
    )
}

#[cfg(not(unix))]
fn current_hour() -> String {
    let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return String::from("--:--:--");
    };
    ui::styles::fmt_hms(std::time::Duration::from_secs(
        now.as_secs() % (24 * 60 * 60),
    ))
}

#[cfg(unix)]
fn current_date() -> String {
    let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return String::from("---");
    };
    let timestamp = now.as_secs() as libc::time_t;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
    let local = unsafe {
        if libc::localtime_r(&timestamp, local.as_mut_ptr()).is_null() {
            return String::from("---");
        }
        local.assume_init()
    };
    unsafe {
        libc::setlocale(libc::LC_TIME, system_locale_cstr().as_ptr());
        let mut wday_buf = [0u8; 32];
        libc::strftime(
            wday_buf.as_mut_ptr() as *mut libc::c_char,
            wday_buf.len(),
            b"%A\0".as_ptr() as *const libc::c_char,
            &local,
        );
        let wday_len = wday_buf.iter().position(|&b| b == 0).unwrap_or(0);
        let mut month_buf = [0u8; 32];
        libc::strftime(
            month_buf.as_mut_ptr() as *mut libc::c_char,
            month_buf.len(),
            b"%B\0".as_ptr() as *const libc::c_char,
            &local,
        );
        let month_len = month_buf.iter().position(|&b| b == 0).unwrap_or(0);
        let wday = String::from_utf8_lossy(&wday_buf[..wday_len]).to_uppercase();
        let month = String::from_utf8_lossy(&month_buf[..month_len]).to_uppercase();
        format!(
            "{} {} {} {}",
            wday,
            local.tm_mday,
            month,
            local.tm_year + 1900
        )
    }
}

#[cfg(not(unix))]
fn current_date() -> String {
    String::from("---")
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
    current_queue_player_id: audio::PlayerId,
    selected_queue_index: Option<usize>,
    autodj_enabled: bool,
    deck_soft_stopping: bool,
    previewing_queue_id: Option<i32>,
    search_tracks: Vec<db::SearchTrack>,
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
    aux_slots: Vec<Option<LoadedTrack>>,
    aux_loops: Vec<bool>,
    app_config: db::AppConfig,
    dialog: Option<Dialog>,
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
        let mut queue_entries = Vec::new();
        let mut app_config = db::AppConfig::default();

        let (db, status) = match db::Database::connect_from_file(&db_config_path()) {
            Ok(db) => {
                let mut warnings = Vec::new();

                match db.load_config() {
                    Ok(cfg) => app_config = cfg,
                    Err(error) => warnings.push(format!("config: {error}")),
                }

                match db.search_tracks() {
                    Ok(tracks) => search_tracks = tracks,
                    Err(error) => warnings.push(format!("tracks: {error}")),
                }

                match db.queue_entries() {
                    Ok(entries) => queue_entries = entries,
                    Err(error) => warnings.push(format!("queue: {error}")),
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
            current_queue_player_id: audio::PlayerId::QueueA,
            selected_queue_index: None,
            autodj_enabled: app_config.auto_mix_on_start,
            deck_soft_stopping: false,
            previewing_queue_id: None,
            search_tracks,
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
            aux_slots: vec![None; 3],
            aux_loops: vec![false; 3],
            app_config,
            dialog: None,
        };
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
enum Dialog {
    SaveInstantPage {
        name: String,
    },
    EditConfig {
        auto_mix_on_start: bool,
        auto_play_on_start: bool,
        preload: String,
        fade_out_duration_ms: String,
        stop_fade_duration_ms: String,
    },
    EditDbConfig {
        host: String,
        port: String,
        database: String,
        user: String,
        password: String,
    },
}

#[derive(Debug, Clone)]
enum DbField {
    Host,
    Port,
    Database,
    User,
    Password,
}

#[derive(Debug, Clone)]
enum ConfigField {
    AutoMixOnStart,
    AutoPlayOnStart,
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
    InstantStop,
    InstantSave,
    InstantSaveNameChanged(String),
    InstantSaveConfirm,
    DialogCancel,
    InstantNewPage,
    InstantDeletePage,
    InstantPreviousPage,
    InstantNextPage,
    ToggleAutoDj,
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
    AuxPlay(usize),
    AuxStop(usize),
    AuxToggleLoop(usize),
    MeterTick,
    ClockTick,
    DbConfigOpen,
    DbConfigFieldChanged(DbField, String),
    DbConfigSave,
    DbConfigConnected(Result<db::SharedDatabase, String>),
    ConfigOpen,
    ConfigToggle(ConfigField),
    ConfigPreloadChanged(String),
    ConfigFadeOutDurationChanged(String),
    ConfigStopFadeDurationChanged(String),
    ConfigSave,
    ConfigSaved(Result<(), String>),
}

// ── Update ────────────────────────────────────────────────────────────────────

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::WindowClosed(window_id) => {
                let closed = self.windows.remove(&window_id);
                match closed {
                    Some(WindowKind::Main) => {
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
                        self.audio
                            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Load { path, cue_in });
                        self.audio
                            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Play);
                    }
                }
                Task::none()
            }

            Message::ShowSearch => {
                self.instant_view = InstantView::Search;
                Task::none()
            }
            Message::ShowInstantPlayers => {
                self.instant_view = InstantView::InstantPlayers;
                Task::none()
            }
            Message::SearchChanged(value) => {
                self.search_query = value;
                self.search_page_start = 0;
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
                Task::none()
            }
            Message::SubcategorySelected(value) => {
                self.search_subcategory = value;
                self.search_page_start = 0;
                Task::none()
            }
            Message::GenreSelected(value) => {
                self.search_genre = value;
                self.search_page_start = 0;
                Task::none()
            }
            Message::SearchRowSelected(track_id) => {
                self.selected_search_track_id = Some(track_id);
                Task::none()
            }
            Message::SearchFirstPage => {
                self.search_page_start = 0;
                Task::none()
            }
            Message::SearchPreviousPage => {
                self.search_page_start = self.search_page_start.saturating_sub(SEARCH_PAGE_SIZE);
                Task::none()
            }
            Message::SearchNextPage => {
                let last_start = self.last_search_page_start();
                self.search_page_start =
                    (self.search_page_start + SEARCH_PAGE_SIZE).min(last_start);
                Task::none()
            }
            Message::SearchLastPage => {
                self.search_page_start = self.last_search_page_start();
                Task::none()
            }
            Message::OpenTrackPicker(target) => {
                let (window_id, open) = window::open(picker_window_settings());
                self.windows.insert(
                    window_id,
                    WindowKind::TrackPicker(TrackPickerState::new(target)),
                );
                open.map(|_| Message::NoOp)
            }
            Message::PickerSearchChanged(window_id, value) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.search_query = value;
                    picker.page_start = 0;
                }
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
                Task::none()
            }
            Message::PickerSubcategorySelected(window_id, value) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.search_subcategory = value;
                    picker.page_start = 0;
                }
                Task::none()
            }
            Message::PickerGenreSelected(window_id, value) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.search_genre = value;
                    picker.page_start = 0;
                }
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
                Task::none()
            }
            Message::PickerPreviousPage(window_id) => {
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.page_start = picker.page_start.saturating_sub(SEARCH_PAGE_SIZE);
                }
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
                Task::none()
            }
            Message::PickerPreviewPlay(window_id) => {
                let selected_id = self
                    .track_picker(window_id)
                    .and_then(|picker| picker.selected_track_id);
                if let Some(id) = selected_id {
                    if let Some(path) = self.search_track_path(id) {
                        let cue_in = self.search_track_cue_in(id);
                        self.audio
                            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Load { path, cue_in });
                        self.audio
                            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Play);
                    }
                }
                Task::none()
            }
            Message::InstantSlotPressed(index) => {
                self.play_instant_slot(index);
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
                        self.audio
                            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Load { path, cue_in });
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
                if let Some(db) = &self.db {
                    match db.clear_queue() {
                        Ok(()) => {
                            self.queue_entries.clear();
                            self.selected_queue_index = None;
                            self.clear_preloaded_queue_status();
                        }
                        Err(e) => self.status = format!("Queue clear failed: {e}"),
                    }
                }
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
                let (host, port, database, user, password) =
                    if let Ok(raw) = std::fs::read_to_string(db_config_path()) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                            (
                                val["host"].as_str().unwrap_or("localhost").to_string(),
                                val["port"].as_u64().unwrap_or(5432).to_string(),
                                val["database"].as_str().unwrap_or("").to_string(),
                                val["user"].as_str().unwrap_or("").to_string(),
                                val["password"].as_str().unwrap_or("").to_string(),
                            )
                        } else {
                            (
                                "localhost".into(),
                                "5432".into(),
                                String::new(),
                                String::new(),
                                String::new(),
                            )
                        }
                    } else {
                        (
                            "localhost".into(),
                            "5432".into(),
                            String::new(),
                            String::new(),
                            String::new(),
                        )
                    };
                self.dialog = Some(Dialog::EditDbConfig {
                    host,
                    port,
                    database,
                    user,
                    password,
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
                }) = &mut self.dialog
                {
                    match field {
                        DbField::Host => *host = value,
                        DbField::Port => *port = value,
                        DbField::Database => *database = value,
                        DbField::User => *user = value,
                        DbField::Password => *password = value,
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
                }) = &self.dialog
                {
                    let config = serde_json::json!({
                        "host": host,
                        "port": port.parse::<u16>().unwrap_or(5432),
                        "database": database,
                        "user": user,
                        "password": password,
                    });
                    let path = db_config_path();
                    match serde_json::to_string_pretty(&config)
                        .map_err(|e| e.to_string())
                        .and_then(|json| std::fs::write(&path, json).map_err(|e| e.to_string()))
                    {
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
                        self.db = Some(db);
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

            Message::ConfigOpen => {
                self.dialog = Some(Dialog::EditConfig {
                    auto_mix_on_start: self.app_config.auto_mix_on_start,
                    auto_play_on_start: self.app_config.auto_play_on_start,
                    preload: self.app_config.preload.to_string(),
                    fade_out_duration_ms: self.app_config.fade_out_duration_ms.to_string(),
                    stop_fade_duration_ms: self.app_config.stop_fade_duration_ms.to_string(),
                });
                Task::none()
            }

            Message::ConfigToggle(field) => {
                if let Some(Dialog::EditConfig {
                    auto_mix_on_start,
                    auto_play_on_start,
                    ..
                }) = &mut self.dialog
                {
                    match field {
                        ConfigField::AutoMixOnStart => *auto_mix_on_start = !*auto_mix_on_start,
                        ConfigField::AutoPlayOnStart => *auto_play_on_start = !*auto_play_on_start,
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

            Message::ConfigSave => {
                if let Some(Dialog::EditConfig {
                    auto_mix_on_start,
                    auto_play_on_start,
                    preload,
                    fade_out_duration_ms,
                    stop_fade_duration_ms,
                }) = &self.dialog
                {
                    let cfg = db::AppConfig {
                        auto_mix_on_start: *auto_mix_on_start,
                        auto_play_on_start: *auto_play_on_start,
                        preload: preload.trim().parse::<i32>().unwrap_or(10).max(0),
                        fade_out_duration_ms: fade_out_duration_ms
                            .trim()
                            .parse::<i32>()
                            .unwrap_or(2500)
                            .max(0),
                        stop_fade_duration_ms: stop_fade_duration_ms
                            .trim()
                            .parse::<i32>()
                            .unwrap_or(1000)
                            .max(0),
                    };
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
                }
                Task::none()
            }

            Message::ConfigSaved(result) => {
                if let Err(e) = result {
                    self.status = format!("Config save failed: {e}");
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::keyboard::on_key_press(|key, _modifiers| {
            matches!(key, Key::Named(Named::Space)).then_some(Message::TogglePlay)
        });
        let clock =
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::ClockTick);
        let windows = window::close_events().map(Message::WindowClosed);
        let mut subscriptions = vec![keyboard, clock, windows];

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

        Subscription::batch(subscriptions)
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

    fn set_auto_mix_status(&mut self, status: impl Into<String>) {
        let status = status.into();
        self.auto_mix_status = status.clone();
        self.log_auto_mix_status(&status);
    }

    fn log_auto_mix_status(&mut self, status: &str) {
        let Some(db) = &self.db else {
            return;
        };

        if let Err(error) = db.insert_automix_log(status) {
            self.status = format!("AUTO MIX log insert failed: {error}");
        }
    }

    fn active_queue_play_log_positions(&self) -> HashMap<audio::PlayerId, std::time::Duration> {
        self.active_queue_play_logs
            .keys()
            .filter_map(|&player_id| {
                self.audio
                    .player(player_id)
                    .is_active()
                    .then(|| (player_id, self.audio.player(player_id).snapshot().position))
            })
            .collect()
    }

    fn begin_queue_play_log(&mut self, player_id: audio::PlayerId, track_id: i32) {
        self.active_queue_play_logs
            .insert(player_id, ActiveQueuePlayLog { track_id });
    }

    fn close_finished_queue_play_logs(
        &mut self,
        positions: &HashMap<audio::PlayerId, std::time::Duration>,
    ) {
        for (&player_id, &position) in positions {
            if !self.audio.player(player_id).is_active() {
                self.close_queue_play_log(player_id, position);
            }
        }
    }

    fn close_queue_play_log(
        &mut self,
        player_id: audio::PlayerId,
        played_duration: std::time::Duration,
    ) {
        let Some(active_log) = self.active_queue_play_logs.remove(&player_id) else {
            return;
        };
        let Some(db) = &self.db else {
            return;
        };
        if let Err(error) = db.insert_play_log(active_log.track_id, played_duration) {
            self.status = format!("Play log insert failed: {error}");
        }
    }

    fn load_next_from_queue(&mut self, player_id: audio::PlayerId) {
        if self.play_preloaded_queue_entry(player_id) {
            return;
        }

        if self.queue_entries.is_empty() {
            return;
        }
        let entry = self.queue_entries.remove(0);
        self.adjust_selected_queue_index_after_remove(0);
        self.play_queue_entry(player_id, entry);
    }

    fn apply_startup_playback_config(&mut self) {
        if self.app_config.auto_play_on_start && !self.queue_entries.is_empty() {
            self.play_queue_entry_now(0);
        }
    }

    fn play_queue_entry(&mut self, player_id: audio::PlayerId, entry: db::QueueEntry) {
        if self
            .preloaded_queue_entry
            .as_ref()
            .is_some_and(|preloaded| preloaded.player_id == player_id)
        {
            self.clear_preloaded_queue_status();
        }

        if let Some(track_id) = entry.track_id {
            if let Some(path) = self.search_track_path(track_id) {
                self.close_queue_play_log(
                    player_id,
                    self.audio.player(player_id).snapshot().position,
                );
                self.audio
                    .handle(player_id, audio::PlayerCommand::Load { path, cue_in: entry.cue_in });
                self.audio.handle(player_id, audio::PlayerCommand::Play);
                self.begin_queue_play_log(player_id, track_id);
            }
        }

        self.finalize_queue_entry_launch(player_id, entry);
    }

    fn queue_entry_label(entry: &db::QueueEntry) -> String {
        match (entry.artist_name.trim(), entry.title.trim()) {
            ("", "") => format!("Queue item {}", entry.id),
            ("", title) => title.to_string(),
            (artist, "") => artist.to_string(),
            (artist, title) => format!("{artist} - {title}"),
        }
    }

    fn clear_preloaded_queue_status(&mut self) {
        if self.preloaded_queue_entry.take().is_some() && self.autodj_enabled {
            self.set_auto_mix_status("Waiting");
        }
    }

    fn preload_next_queue_entry(&mut self, player_id: audio::PlayerId) {
        if self.queue_entries.is_empty() || self.audio.player(player_id).is_active() {
            return;
        }

        let Some(entry) = self.queue_entries.first().cloned() else {
            return;
        };

        if self
            .preloaded_queue_entry
            .as_ref()
            .is_some_and(|preloaded| {
                preloaded.player_id == player_id && preloaded.entry.id == entry.id
            })
        {
            return;
        }

        let Some(path) = entry
            .track_id
            .and_then(|track_id| self.search_track_path(track_id))
        else {
            return;
        };

        self.audio
            .handle(player_id, audio::PlayerCommand::Load { path, cue_in: entry.cue_in });
        self.set_auto_mix_status(format!(
            "Track {} has been preloaded.",
            Self::queue_entry_label(&entry)
        ));
        self.preloaded_queue_entry = Some(PreloadedQueueEntry { player_id, entry });
    }

    fn play_preloaded_queue_entry(&mut self, player_id: audio::PlayerId) -> bool {
        let Some(preloaded) = self.preloaded_queue_entry.clone() else {
            return false;
        };

        let matches_next_queue_entry = self
            .queue_entries
            .first()
            .is_some_and(|entry| entry.id == preloaded.entry.id);

        if preloaded.player_id != player_id || !matches_next_queue_entry {
            if preloaded.player_id == player_id {
                self.clear_preloaded_queue_status();
            }
            return false;
        }

        self.queue_entries.remove(0);
        self.adjust_selected_queue_index_after_remove(0);
        self.audio.handle(player_id, audio::PlayerCommand::Play);
        if let Some(track_id) = preloaded.entry.track_id {
            self.begin_queue_play_log(player_id, track_id);
        }
        self.set_auto_mix_status(format!(
            "Track {} has started.",
            Self::queue_entry_label(&preloaded.entry)
        ));
        self.finalize_queue_entry_launch(player_id, preloaded.entry);
        self.preloaded_queue_entry = None;
        true
    }

    fn finalize_queue_entry_launch(&mut self, player_id: audio::PlayerId, entry: db::QueueEntry) {
        if self.previewing_queue_id == Some(entry.id) {
            self.stop_preview();
            self.previewing_queue_id = None;
        }

        if let Some(db) = &self.db {
            if let Err(e) = db.delete_queue_entry(entry.id) {
                self.status = format!("Queue entry delete failed: {e}");
            }
        }

        self.queue_player_entries.insert(player_id, entry.clone());
        self.current_queue_player_id = player_id;
        self.current_queue_entry = Some(entry);
    }

    fn play_queue_entry_now(&mut self, index: usize) {
        if index >= self.queue_entries.len() {
            return;
        }

        let player_id = self.queue_player_id_for_immediate_launch();
        let entry = self.queue_entries.remove(index);
        self.adjust_selected_queue_index_after_remove(index);
        self.play_queue_entry(player_id, entry);
        self.fade_out_previous_queue_players(player_id);
    }

    fn any_queue_active(&self) -> bool {
        QUEUE_PLAYER_IDS
            .iter()
            .any(|&player_id| self.audio.player(player_id).is_active())
    }

    fn queue_active_flags(&self) -> [bool; 2] {
        QUEUE_PLAYER_IDS.map(|player_id| self.audio.player(player_id).is_active())
    }

    fn next_queue_player_id(&self, current: audio::PlayerId) -> audio::PlayerId {
        match current {
            audio::PlayerId::QueueA => audio::PlayerId::QueueB,
            audio::PlayerId::QueueB => audio::PlayerId::QueueA,
            _ => audio::PlayerId::QueueA,
        }
    }

    fn queue_player_id_for_immediate_launch(&self) -> audio::PlayerId {
        if !self.any_queue_active() {
            self.current_queue_player_id
        } else {
            let next_player_id = self.next_queue_player_id(self.current_queue_player_id);
            if !self.audio.player(next_player_id).is_active() {
                next_player_id
            } else if !self.audio.player(self.current_queue_player_id).is_active() {
                self.current_queue_player_id
            } else {
                next_player_id
            }
        }
    }

    fn configured_fade_out_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.app_config.fade_out_duration_ms.max(0) as u64)
    }

    fn fade_out_previous_queue_players(&mut self, keep_player_id: audio::PlayerId) {
        let fade_out_duration = self.configured_fade_out_duration();
        let players_to_fade: Vec<_> = QUEUE_PLAYER_IDS
            .into_iter()
            .filter(|&player_id| {
                player_id != keep_player_id && self.audio.player(player_id).is_active()
            })
            .collect();

        for player_id in players_to_fade {
            if fade_out_duration.is_zero() {
                self.close_queue_play_log(
                    player_id,
                    self.audio.player(player_id).snapshot().position,
                );
                self.audio.handle(player_id, audio::PlayerCommand::Stop);
                self.queue_player_entries.remove(&player_id);
            } else {
                self.audio
                    .handle(player_id, audio::PlayerCommand::FadeOut(fade_out_duration));
            }
        }
    }

    fn adjust_selected_queue_index_after_remove(&mut self, removed_index: usize) {
        let Some(selected_index) = self.selected_queue_index else {
            return;
        };

        self.selected_queue_index = if self.queue_entries.is_empty() {
            None
        } else if selected_index == removed_index {
            Some(removed_index.min(self.queue_entries.len() - 1))
        } else if selected_index > removed_index {
            Some(selected_index - 1)
        } else {
            Some(selected_index)
        };
    }

    fn stop_queue_players(&mut self) {
        for player_id in QUEUE_PLAYER_IDS {
            self.close_queue_play_log(player_id, self.audio.player(player_id).snapshot().position);
            self.audio.handle(player_id, audio::PlayerCommand::Stop);
        }
        self.queue_player_entries.clear();
        self.preloaded_queue_entry = None;
        if self.autodj_enabled {
            self.set_auto_mix_status("Stopped");
        } else {
            self.set_auto_mix_status("Disabled");
        }
        self.current_queue_entry = None;
        self.current_queue_player_id = audio::PlayerId::QueueA;
    }

    fn sync_queue_players(&mut self, was_active: [bool; 2]) {
        let mut current_finished = None;

        for (index, player_id) in QUEUE_PLAYER_IDS.into_iter().enumerate() {
            if was_active[index] && !self.audio.player(player_id).is_active() {
                self.queue_player_entries.remove(&player_id);
                if self.current_queue_player_id == player_id {
                    current_finished = Some(player_id);
                }
            }
        }

        if let Some(player_id) = current_finished {
            if self.deck_soft_stopping {
                self.deck_soft_stopping = false;
                for pid in QUEUE_PLAYER_IDS {
                    if self.audio.player(pid).is_active() {
                        self.audio.handle(pid, audio::PlayerCommand::Stop);
                    }
                }
                self.queue_player_entries.clear();
                self.current_queue_player_id = audio::PlayerId::QueueA;
                if self.autodj_enabled {
                    self.set_auto_mix_status("Stopped");
                } else {
                    self.set_auto_mix_status("Disabled");
                }
            } else if self.autodj_enabled && !self.any_queue_active() {
                let next_player_id = self.next_queue_player_id(player_id);
                self.load_next_from_queue(next_player_id);
            }
        }

        self.refresh_current_queue_entry();
    }

    fn sync_auto_mix(&mut self) {
        if !self.autodj_enabled || self.queue_entries.is_empty() {
            return;
        }

        let player_id = self.current_queue_player_id;
        if !self.audio.player(player_id).is_playing() {
            return;
        }

        let Some(entry) = self.queue_player_entries.get(&player_id) else {
            return;
        };

        let trigger = auto_mix_trigger(entry);
        if trigger == std::time::Duration::ZERO {
            return;
        }

        let snapshot = self.audio.player(player_id).snapshot();
        let next_player_id = self.next_queue_player_id(player_id);
        let preload_at = trigger.saturating_sub(std::time::Duration::from_secs(
            self.app_config.preload.max(0) as u64,
        ));

        if snapshot.position >= preload_at && !self.audio.player(next_player_id).is_active() {
            self.preload_next_queue_entry(next_player_id);
        }

        if snapshot.position < trigger {
            return;
        }

        if self.audio.player(next_player_id).is_active() {
            return;
        }

        self.load_next_from_queue(next_player_id);
    }

    fn refresh_current_queue_entry(&mut self) {
        if self.audio.player(self.current_queue_player_id).is_active() {
            if let Some(entry) = self
                .queue_player_entries
                .get(&self.current_queue_player_id)
                .cloned()
            {
                self.current_queue_entry = Some(entry);
                return;
            }
        }

        for player_id in QUEUE_PLAYER_IDS {
            if self.audio.player(player_id).is_active() {
                if let Some(entry) = self.queue_player_entries.get(&player_id).cloned() {
                    self.current_queue_player_id = player_id;
                    self.current_queue_entry = Some(entry);
                    return;
                }
            }
        }

        self.current_queue_entry = None;
    }

    fn search_track_path(&self, track_id: i32) -> Option<PathBuf> {
        let path = self
            .search_tracks
            .iter()
            .find(|track| track.id == track_id)?
            .path
            .trim();

        (!path.is_empty()).then(|| PathBuf::from(path))
    }

    fn search_track_cue_in(&self, track_id: i32) -> std::time::Duration {
        self.search_tracks
            .iter()
            .find(|track| track.id == track_id)
            .map(|track| track.cue_in)
            .unwrap_or_default()
    }

    fn loaded_track(&self, track_id: i32) -> Option<LoadedTrack> {
        let track = self
            .search_tracks
            .iter()
            .find(|track| track.id == track_id)?;
        Some(LoadedTrack {
            id: track.id,
            artist: track.artist_name.clone(),
            title: track.title.clone(),
            duration: track.duration,
            cue_in: track.cue_in,
            path: PathBuf::from(track.path.trim()),
        })
    }

    fn load_instant_pages_from_db(&mut self) {
        let Some(db) = self.db.clone() else {
            return;
        };

        match db.instant_pages() {
            Ok(pages) if pages.is_empty() => {
                self.instant_pages = vec![InstantPage::default()];
                self.active_instant_page = 0;
                self.instant_slots = vec![None; 10];
            }
            Ok(pages) => {
                self.instant_pages = pages
                    .into_iter()
                    .map(|page| InstantPage {
                        id: Some(page.id),
                        name: page.name,
                    })
                    .collect();
                self.active_instant_page =
                    self.active_instant_page.min(self.instant_pages.len() - 1);
                self.load_active_instant_slots();
            }
            Err(error) => self.status = format!("Instant pages unavailable: {error}"),
        }
    }

    fn load_active_instant_slots(&mut self) {
        self.stop_instant();
        self.instant_slots = vec![None; 10];

        let Some(db) = self.db.clone() else {
            return;
        };
        let Some(page_id) = self.active_instant_page_id() else {
            return;
        };

        match db.instant_slots(page_id) {
            Ok(slots) => {
                for slot in slots {
                    if slot.slot_index < self.instant_slots.len() {
                        self.instant_slots[slot.slot_index] = self.loaded_track(slot.track_id);
                    }
                }
            }
            Err(error) => self.status = format!("Instant slots unavailable: {error}"),
        }
    }

    fn active_instant_page_id(&self) -> Option<i32> {
        self.instant_pages
            .get(self.active_instant_page)
            .and_then(|page| page.id)
    }

    pub(crate) fn active_instant_page_name(&self) -> String {
        self.instant_pages
            .get(self.active_instant_page)
            .map(|page| page.name.clone())
            .unwrap_or_else(|| String::from("Default"))
    }

    fn open_save_instant_dialog(&mut self) {
        let current = self.active_instant_page_name();
        let name = if current == "Default" {
            String::new()
        } else {
            current
        };
        self.dialog = Some(Dialog::SaveInstantPage { name });
    }

    fn new_instant_page(&mut self) {
        self.stop_instant();
        self.instant_pages.push(InstantPage::default());
        self.active_instant_page = self.instant_pages.len() - 1;
        self.instant_slots = vec![None; 10];
        self.dialog = Some(Dialog::SaveInstantPage {
            name: String::new(),
        });
        self.status = String::from("New instant page");
    }

    fn save_instant_page(&mut self) {
        let Some(db) = self.db.clone() else {
            self.status = String::from("Disconnected (instant page not saved)");
            return;
        };

        let name = match &self.dialog {
            Some(Dialog::SaveInstantPage { name }) => {
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    "Default".to_string()
                } else {
                    trimmed.chars().take(64).collect()
                }
            }
            None | Some(Dialog::EditDbConfig { .. }) | Some(Dialog::EditConfig { .. }) => {
                self.active_instant_page_name()
            }
        };

        let page_id = match self.active_instant_page_id() {
            Some(page_id) => {
                if let Err(error) = db.update_instant_page_name(page_id, &name) {
                    self.status = format!("Instant page save failed: {error}");
                    return;
                }
                page_id
            }
            None => match db.insert_instant_page(&name) {
                Ok(page_id) => page_id,
                Err(error) => {
                    self.status = format!("Instant page creation failed: {error}");
                    return;
                }
            },
        };

        if let Err(error) = db.clear_instant_slots(page_id) {
            self.status = format!("Instant slots clear failed: {error}");
            return;
        }

        for (slot_index, slot) in self.instant_slots.iter().enumerate() {
            if let Some(track) = slot {
                if let Err(error) = db.insert_instant_slot(page_id, slot_index, track.id) {
                    self.status = format!("Instant slot save failed: {error}");
                    return;
                }
            }
        }

        if let Some(page) = self.instant_pages.get_mut(self.active_instant_page) {
            page.id = Some(page_id);
            page.name = name.clone();
        } else {
            self.instant_pages.push(InstantPage {
                id: Some(page_id),
                name: name.clone(),
            });
            self.active_instant_page = self.instant_pages.len() - 1;
        }

        self.dialog = None;
        self.status = format!("Instant page saved: {name}");
    }

    fn delete_active_instant_page(&mut self) {
        let Some(page_id) = self.active_instant_page_id() else {
            self.instant_slots = vec![None; 10];
            self.status = String::from("Instant page is empty");
            return;
        };
        let Some(db) = self.db.clone() else {
            self.status = String::from("Disconnected (instant page not deleted)");
            return;
        };

        if let Err(error) = db.delete_instant_page(page_id) {
            self.status = format!("Instant page delete failed: {error}");
            return;
        }

        self.status = format!("Instant page deleted: {}", self.active_instant_page_name());
        self.load_instant_pages_from_db();
        if self.instant_pages.is_empty() {
            self.instant_pages = vec![InstantPage::default()];
            self.active_instant_page = 0;
            self.instant_slots = vec![None; 10];
        }
    }

    fn show_previous_instant_page(&mut self) {
        if self.instant_pages.is_empty() {
            return;
        }
        self.active_instant_page = if self.active_instant_page == 0 {
            self.instant_pages.len() - 1
        } else {
            self.active_instant_page - 1
        };
        self.load_active_instant_slots();
    }

    fn show_next_instant_page(&mut self) {
        if self.instant_pages.is_empty() {
            return;
        }
        self.active_instant_page = (self.active_instant_page + 1) % self.instant_pages.len();
        self.load_active_instant_slots();
    }

    fn assign_loaded_track(&mut self, target: PickerTarget, track: LoadedTrack) {
        match target {
            PickerTarget::Instant(index) => {
                if let Some(slot) = self.instant_slots.get_mut(index) {
                    *slot = Some(track);
                }
            }
            PickerTarget::Aux(index) => {
                if let Some(slot) = self.aux_slots.get_mut(index) {
                    *slot = Some(track);
                }
            }
        }
    }

    fn stop_preview(&mut self) {
        self.audio
            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Stop);
    }

    fn play_instant_slot(&mut self, index: usize) {
        let Some(track) = self.instant_slots.get(index).and_then(Option::as_ref) else {
            return;
        };
        let path = track.path.clone();
        let cue_in = track.cue_in;

        self.audio
            .handle(INSTANT_PLAYER_ID, audio::PlayerCommand::Load { path, cue_in });
        self.audio
            .handle(INSTANT_PLAYER_ID, audio::PlayerCommand::Play);
        self.active_instant_slot = Some(index);
    }

    fn stop_instant(&mut self) {
        self.audio
            .handle(INSTANT_PLAYER_ID, audio::PlayerCommand::Stop);
        self.active_instant_slot = None;
    }

    fn sync_instant_active_slot(&mut self) {
        if self.active_instant_slot.is_some() && !self.audio.player(INSTANT_PLAYER_ID).is_active() {
            self.active_instant_slot = None;
        }
    }

    pub(crate) fn instant_duration_display(
        &self,
        slot_index: usize,
        total: std::time::Duration,
    ) -> std::time::Duration {
        if self.active_instant_slot == Some(slot_index) {
            let elapsed = self
                .audio
                .player(INSTANT_PLAYER_ID)
                .snapshot()
                .position
                .min(total);
            total.saturating_sub(elapsed)
        } else {
            total
        }
    }

    fn insert_selected_search_into_queue(&mut self) {
        let Some(track_id) = self.selected_search_track_id else {
            return;
        };
        let Some(track) = self.search_tracks.iter().find(|t| t.id == track_id) else {
            return;
        };
        let new_entry = db::QueueEntry {
            id: 0,
            track_id: Some(track.id),
            artist_name: track.artist_name.clone(),
            title: track.title.clone(),
            duration: track.duration,
            intro: track.intro,
            outro: track.outro,
            cue_in: std::time::Duration::ZERO,
            cue_out: track.cue_out,
            scheduled_at: None,
            priority: 0,
            fixed_time: false,
        };

        let insert_at = self
            .selected_queue_index
            .map(|i| i + 1)
            .unwrap_or(self.queue_entries.len());

        let Some(db) = &self.db else {
            return;
        };
        match db.insert_queue_entry(track_id) {
            Ok(new_id) => {
                self.clear_preloaded_queue_status();
                let mut entry = new_entry;
                entry.id = new_id;
                let insert_at = insert_at.min(self.queue_entries.len());
                self.queue_entries.insert(insert_at, entry);
                self.selected_queue_index = Some(insert_at);
            }
            Err(e) => {
                self.status = format!("Queue insert failed: {e}");
            }
        }
    }

    fn replace_selected_queue_entry(&mut self) {
        let Some(track_id) = self.selected_search_track_id else {
            return;
        };
        let Some(queue_index) = self.selected_queue_index else {
            return;
        };
        let Some(track) = self.search_tracks.iter().find(|t| t.id == track_id) else {
            return;
        };
        let (new_artist, new_title, new_duration, new_intro, new_cue_out) = (
            track.artist_name.clone(),
            track.title.clone(),
            track.duration,
            track.intro,
            track.cue_out,
        );
        self.clear_preloaded_queue_status();
        let Some(entry) = self.queue_entries.get_mut(queue_index) else {
            return;
        };
        let queue_id = entry.id;

        let Some(db) = &self.db else {
            return;
        };
        match db.replace_queue_entry(queue_id, track_id) {
            Ok(()) => {
                entry.track_id = Some(track_id);
                entry.artist_name = new_artist;
                entry.title = new_title;
                entry.duration = new_duration;
                entry.intro = new_intro;
                entry.cue_in = std::time::Duration::ZERO;
                entry.cue_out = new_cue_out;
            }
            Err(e) => self.status = format!("Queue replace failed: {e}"),
        }
    }

    fn remove_selected_queue_entry(&mut self) {
        let Some(queue_index) = self.selected_queue_index else {
            return;
        };
        let Some(entry) = self.queue_entries.get(queue_index) else {
            return;
        };
        let queue_id = entry.id;

        let Some(db) = &self.db else {
            return;
        };
        match db.delete_queue_entry(queue_id) {
            Ok(()) => {
                self.clear_preloaded_queue_status();
                self.queue_entries.remove(queue_index);
                self.selected_queue_index = if self.queue_entries.is_empty() {
                    None
                } else {
                    Some(queue_index.min(self.queue_entries.len() - 1))
                };
            }
            Err(e) => self.status = format!("Queue entry delete failed: {e}"),
        }
    }

    fn aux_player_id(index: usize) -> Option<audio::PlayerId> {
        AUX_PLAYER_IDS.get(index).copied()
    }

    fn aux_active_flags(&self) -> [bool; 3] {
        AUX_PLAYER_IDS.map(|id| self.audio.player(id).is_active())
    }

    fn aux_is_active(&self, index: usize) -> bool {
        Self::aux_player_id(index).is_some_and(|id| self.audio.player(id).is_active())
    }

    fn aux_progress_parts(&self, index: usize) -> (u16, u16) {
        let Some(player_id) = Self::aux_player_id(index) else {
            return (1, 1000);
        };
        let snapshot = self.audio.player(player_id).snapshot();
        let filled = snapshot
            .duration
            .map(|duration| {
                let total = duration.as_millis().max(1);
                ((snapshot.position.as_millis() * 1000) / total).min(1000) as u16
            })
            .unwrap_or(0)
            .max(1);
        let empty = 1000_u16.saturating_sub(filled).max(1);
        (filled, empty)
    }

    fn aux_timing(
        &self,
        index: usize,
    ) -> (
        std::time::Duration,
        std::time::Duration,
        std::time::Duration,
    ) {
        let total = self
            .aux_slots
            .get(index)
            .and_then(Option::as_ref)
            .map(|track| track.duration)
            .unwrap_or_default();
        let elapsed = Self::aux_player_id(index)
            .map(|player_id| self.audio.player(player_id).snapshot().position)
            .unwrap_or_default()
            .min(total);
        let remaining = total.saturating_sub(elapsed);

        (elapsed, remaining, total)
    }

    fn play_aux_slot(&mut self, index: usize) {
        let Some(player_id) = Self::aux_player_id(index) else {
            return;
        };
        let Some(track) = self.aux_slots.get(index).and_then(Option::as_ref) else {
            return;
        };
        let path = track.path.clone();
        let cue_in = track.cue_in;

        self.audio
            .handle(player_id, audio::PlayerCommand::Load { path, cue_in });
        self.audio.handle(player_id, audio::PlayerCommand::Play);
    }

    fn stop_aux_slot(&mut self, index: usize) {
        if let Some(player_id) = Self::aux_player_id(index) {
            self.audio.handle(player_id, audio::PlayerCommand::Stop);
        }
    }

    fn toggle_aux_loop(&mut self, index: usize) {
        if let Some(looping) = self.aux_loops.get_mut(index) {
            *looping = !*looping;
        }
    }

    fn sync_aux_loops(&mut self, was_active: [bool; 3]) {
        for (index, was_active) in was_active.into_iter().enumerate() {
            if !was_active || self.aux_is_active(index) {
                continue;
            }
            if self.aux_loops.get(index).copied().unwrap_or(false) {
                self.play_aux_slot(index);
            }
        }
    }

    pub(crate) fn search_track_matches(&self, track: &db::SearchTrack) -> bool {
        let query = self.search_query.trim().to_lowercase();
        let matches_query = query.is_empty()
            || track.artist_name.to_lowercase().contains(&query)
            || track.title.to_lowercase().contains(&query);

        matches_query
            && self.search_category.matches_id(track.category_id)
            && self.search_subcategory.matches_id(track.subcategory_id)
            && self.search_genre.matches_id(track.genre_id)
    }

    pub(crate) fn picker_track_matches(
        &self,
        picker: &TrackPickerState,
        track: &db::SearchTrack,
    ) -> bool {
        let query = picker.search_query.trim().to_lowercase();
        let matches_query = query.is_empty()
            || track.artist_name.to_lowercase().contains(&query)
            || track.title.to_lowercase().contains(&query);

        matches_query
            && picker.search_category.matches_id(track.category_id)
            && picker.search_subcategory.matches_id(track.subcategory_id)
            && picker.search_genre.matches_id(track.genre_id)
    }

    pub(crate) fn visible_subcategories(&self) -> Vec<db::FilterOption> {
        self.search_subcategories
            .iter()
            .filter(|option| {
                option.id.is_none() || self.search_category.matches_id(option.parent_id)
            })
            .cloned()
            .collect()
    }

    pub(crate) fn picker_visible_subcategories(
        &self,
        picker: &TrackPickerState,
    ) -> Vec<db::FilterOption> {
        self.search_subcategories
            .iter()
            .filter(|option| {
                option.id.is_none() || picker.search_category.matches_id(option.parent_id)
            })
            .cloned()
            .collect()
    }

    pub(crate) fn filtered_search_total(&self) -> usize {
        self.search_tracks
            .iter()
            .filter(|track| self.search_track_matches(track))
            .count()
    }

    pub(crate) fn filtered_picker_total(&self, picker: &TrackPickerState) -> usize {
        self.search_tracks
            .iter()
            .filter(|track| self.picker_track_matches(picker, track))
            .count()
    }

    fn last_search_page_start(&self) -> usize {
        let total = self.filtered_search_total();
        if total == 0 {
            0
        } else {
            ((total - 1) / SEARCH_PAGE_SIZE) * SEARCH_PAGE_SIZE
        }
    }

    fn last_picker_page_start(&self, picker: &TrackPickerState) -> usize {
        let total = self.filtered_picker_total(picker);
        if total == 0 {
            0
        } else {
            ((total - 1) / SEARCH_PAGE_SIZE) * SEARCH_PAGE_SIZE
        }
    }
}

// ── View ──────────────────────────────────────────────────────────────────────

impl App {
    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(WindowKind::TrackPicker(picker)) = self.windows.get(&window_id) {
            return self.track_picker_window(window_id, picker);
        }

        let content: Element<_> = column![
            self.deck_header(false),
            self.progress_strip(),
            responsive(|size| self.main_stage(size.width < 980.0)),
            self.footer_bar(),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        if self.dialog.is_some() {
            stack([content, self.dialog_overlay()]).into()
        } else {
            content
        }
    }

    fn main_stage(&self, compact: bool) -> Element<'_, Message> {
        let content: Element<_> = if compact {
            column![
                self.queue_panel(),
                self.instant_panel(),
                self.aux_players_panel()
            ]
            .spacing(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            row![
                container(self.queue_panel())
                    .width(Length::FillPortion(7))
                    .height(Length::Fill),
                column![
                    container(self.instant_panel())
                        .width(Length::Fill)
                        .height(Length::FillPortion(7)),
                    container(self.aux_players_panel())
                        .width(Length::Fill)
                        .height(Length::FillPortion(3)),
                ]
                .spacing(4)
                .width(Length::FillPortion(12))
                .height(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(block_style(rgb(7, 11, 13)))
            .into()
    }

    fn footer_bar(&self) -> Element<'_, Message> {
        let db_status = self.db_status_display();
        let status_color = if self.db.is_some() {
            rgb(221, 230, 237)
        } else {
            rgb(255, 190, 120)
        };
        let db_icon_color = if self.db.is_some() {
            rgb(100, 140, 170)
        } else {
            rgb(255, 160, 80)
        };
        let icon_btn = |icon: Bootstrap, msg: Option<Message>, color: Color| {
            let t = text(icon.to_string())
                .font(BOOTSTRAP_FONT)
                .size(14)
                .style(text_color(color));
            let b = button(t).padding([0, 10]).style(|_, _| button::Style {
                background: None,
                ..Default::default()
            });
            if let Some(m) = msg {
                b.on_press(m)
            } else {
                b
            }
        };

        let active_color = rgb(100, 140, 170);
        let inactive_color = rgb(70, 90, 105);

        let cfg_btn = icon_btn(
            Bootstrap::GearFill,
            self.db.is_some().then_some(Message::ConfigOpen),
            if self.db.is_some() {
                active_color
            } else {
                inactive_color
            },
        );
        let db_btn = icon_btn(
            Bootstrap::DatabaseFill,
            Some(Message::DbConfigOpen),
            db_icon_color,
        );
        let auto_mix_color = if self.autodj_enabled {
            rgb(221, 230, 237)
        } else {
            rgb(125, 154, 171)
        };
        let section_label =
            |label: &'static str| text(label).size(12).style(text_color(rgb(160, 180, 195)));

        container(
            row![
                container(
                    row![
                        section_label("DB:"),
                        text(db_status).size(13).style(text_color(status_color))
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .width(Length::FillPortion(5))
                .height(Length::Fill)
                .padding([0, 12])
                .center_y(Length::Fill),
                container(Space::with_width(Length::Fixed(1.0)))
                    .width(Length::Fixed(1.0))
                    .height(Length::Fill)
                    .style(block_style(rgb(37, 54, 64))),
                container(
                    row![
                        section_label("AUTO MIX:"),
                        text(self.auto_mix_status.clone())
                            .size(13)
                            .style(text_color(auto_mix_color))
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .width(Length::FillPortion(7))
                .height(Length::Fill)
                .padding([0, 12])
                .center_y(Length::Fill),
                row![cfg_btn, db_btn].spacing(0).align_y(Alignment::Center),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .style(block_style(rgb(55, 75, 89)))
        .into()
    }

    fn dialog_overlay(&self) -> Element<'_, Message> {
        let dialog = match &self.dialog {
            Some(Dialog::SaveInstantPage { name }) => container(
                column![
                    text("Save Instant Page")
                        .size(14)
                        .style(text_color(rgb(226, 238, 245))),
                    text_input("Page name", name)
                        .on_input(Message::InstantSaveNameChanged)
                        .on_submit(Message::InstantSaveConfirm)
                        .padding(8)
                        .size(13)
                        .width(Length::Fill),
                    row![
                        Space::with_width(Length::Fill),
                        self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                        self.dialog_button("Save", Message::InstantSaveConfirm, accent_purple()),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ]
                .spacing(12),
            )
            .width(Length::Fixed(420.0))
            .padding(16)
            .style(panel_style(rgb(31, 46, 55), accent_purple())),
            Some(Dialog::EditDbConfig {
                host,
                port,
                database,
                user,
                password,
            }) => {
                let lbl = |s: &'static str| text(s).size(11).style(text_color(rgb(160, 180, 195)));
                let field = |label: &'static str, val: &str, f: DbField| {
                    column![
                        lbl(label),
                        text_input("", val)
                            .on_input(move |v| Message::DbConfigFieldChanged(f.clone(), v))
                            .padding(7)
                            .size(13)
                            .width(Length::Fill),
                    ]
                    .spacing(4)
                };
                container(
                    column![
                        text("Database Settings")
                            .size(14)
                            .style(text_color(rgb(226, 238, 245))),
                        field("Host", host, DbField::Host),
                        field("Port", port, DbField::Port),
                        field("Database", database, DbField::Database),
                        field("User", user, DbField::User),
                        column![
                            lbl("Password"),
                            text_input("", password)
                                .on_input(|v| Message::DbConfigFieldChanged(DbField::Password, v))
                                .secure(true)
                                .padding(7)
                                .size(13)
                                .width(Length::Fill),
                        ]
                        .spacing(4),
                        row![
                            Space::with_width(Length::Fill),
                            self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                            self.dialog_button(
                                "Save & Reconnect",
                                Message::DbConfigSave,
                                accent_purple()
                            ),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(12),
                )
                .width(Length::Fixed(460.0))
                .padding(16)
                .style(panel_style(rgb(31, 46, 55), accent_purple()))
            }
            Some(Dialog::EditConfig {
                auto_mix_on_start,
                auto_play_on_start,
                preload,
                fade_out_duration_ms,
                stop_fade_duration_ms,
            }) => {
                let fieldset_label = container(
                    text("AUTO MIX")
                        .size(11)
                        .style(text_color(rgb(160, 180, 195))),
                )
                .padding([3, 8]);

                let fieldset_body = column![
                    checkbox("Enable AUTO MIX on startup", *auto_mix_on_start)
                        .on_toggle(|_| Message::ConfigToggle(ConfigField::AutoMixOnStart))
                        .size(14)
                        .text_size(13),
                    checkbox("Enable AUTO PLAY on startup", *auto_play_on_start)
                        .on_toggle(|_| Message::ConfigToggle(ConfigField::AutoPlayOnStart))
                        .size(14)
                        .text_size(13),
                    row![
                        text("Preload")
                            .size(13)
                            .style(text_color(rgb(226, 238, 245))),
                        Space::with_width(Length::Fill),
                        text_input("", preload)
                            .on_input(Message::ConfigPreloadChanged)
                            .padding(6)
                            .size(13)
                            .width(Length::Fixed(70.0)),
                        text("s").size(13).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                    row![
                        text("Fade Out Duration")
                            .size(13)
                            .style(text_color(rgb(226, 238, 245))),
                        Space::with_width(Length::Fill),
                        text_input("", fade_out_duration_ms)
                            .on_input(Message::ConfigFadeOutDurationChanged)
                            .padding(6)
                            .size(13)
                            .width(Length::Fixed(70.0)),
                        text("ms").size(13).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                    row![
                        text("Stop Fade Out Duration")
                            .size(13)
                            .style(text_color(rgb(226, 238, 245))),
                        Space::with_width(Length::Fill),
                        text_input("", stop_fade_duration_ms)
                            .on_input(Message::ConfigStopFadeDurationChanged)
                            .padding(6)
                            .size(13)
                            .width(Length::Fixed(70.0)),
                        text("ms").size(13).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                ]
                .spacing(10)
                .padding([10, 12]);

                let fieldset =
                    container(column![fieldset_label, fieldset_body].spacing(0)).style(|_| {
                        container::Style {
                            border: Border {
                                color: rgb(62, 83, 97),
                                width: 1.0,
                                radius: 3.0.into(),
                            },
                            ..Default::default()
                        }
                    });

                container(
                    column![
                        text("Settings")
                            .size(14)
                            .style(text_color(rgb(226, 238, 245))),
                        fieldset,
                        row![
                            Space::with_width(Length::Fill),
                            self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                            self.dialog_button("Save", Message::ConfigSave, accent_purple()),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(16),
                )
                .width(Length::Fixed(380.0))
                .padding(16)
                .style(panel_style(rgb(31, 46, 55), accent_purple()))
            }
            None => container(Space::new(Length::Shrink, Length::Shrink)),
        };

        container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
                ..Default::default()
            })
            .into()
    }

    fn dialog_button(
        &self,
        label: &'static str,
        message: Message,
        bg: Color,
    ) -> Element<'_, Message> {
        button(text(label).size(12))
            .padding([7, 14])
            .on_press(message)
            .style(move |_, status| button::Style {
                background: Some(Background::Color(match status {
                    button::Status::Hovered | button::Status::Pressed => rgb(73, 98, 115),
                    _ => bg,
                })),
                text_color: Color::WHITE,
                border: Border {
                    color: rgb(29, 43, 52),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}
