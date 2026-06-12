use iced::{window, Size};

use crate::app_constants::{METER_DECAY_PER_SECOND, METER_TICK_MS, SEARCH_PAGE_SIZE};
use crate::{db, rest, LoadedTrack};

pub(crate) fn main_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(1240.0, 820.0),
        min_size: Some(Size::new(960.0, 640.0)),
        position: window::Position::Centered,
        exit_on_close_request: false,
        ..window::Settings::default()
    }
}

pub(crate) fn picker_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(980.0, 680.0),
        min_size: Some(Size::new(780.0, 520.0)),
        position: window::Position::Centered,
        ..window::Settings::default()
    }
}

pub(crate) fn audio_peak_to_meter(peak: f32) -> f32 {
    if peak <= 0.000_001 {
        return 0.0;
    }

    let db = 20.0 * peak.clamp(0.000_001, 1.0).log10();
    ((db + 60.0) / 60.0).clamp(0.0, 1.0)
}

pub(crate) fn smooth_meter(current: f32, target: f32) -> f32 {
    if target >= current {
        target
    } else {
        let decay = METER_DECAY_PER_SECOND * METER_TICK_MS as f32 / 1000.0;
        (current - decay).max(target).max(0.0)
    }
}

pub(crate) fn auto_mix_trigger(entry: &db::QueueEntry) -> std::time::Duration {
    if entry.cue_out > std::time::Duration::ZERO && entry.cue_out < entry.duration {
        entry.cue_out
    } else {
        entry.duration
    }
}

pub(crate) fn page_start_for_total(total: usize) -> usize {
    if total == 0 {
        0
    } else {
        ((total - 1) / SEARCH_PAGE_SIZE) * SEARCH_PAGE_SIZE
    }
}

pub(crate) fn login_input_id() -> iced::widget::text_input::Id {
    iced::widget::text_input::Id::new("login_field")
}

pub(crate) fn pass_input_id() -> iced::widget::text_input::Id {
    iced::widget::text_input::Id::new("pass_field")
}

pub(crate) fn duration_ms(duration: std::time::Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

pub(crate) fn queue_track_status(entry: &db::QueueEntry) -> rest::TrackStatus {
    rest::TrackStatus {
        track_id: entry.track_id,
        artist: entry.artist_name.clone(),
        title: entry.title.clone(),
        duration_ms: duration_ms(entry.duration),
    }
}

pub(crate) fn loaded_track_status(track: &LoadedTrack) -> rest::TrackStatus {
    rest::TrackStatus {
        track_id: Some(track.id),
        artist: track.artist.clone(),
        title: track.title.clone(),
        duration_ms: duration_ms(track.duration),
    }
}
