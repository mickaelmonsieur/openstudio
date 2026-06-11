use std::path::PathBuf;

use iced::window;

use crate::{
    db, page_start_for_total, App, LoadedTrack, TrackPickerState, WindowKind, SEARCH_PAGE_SIZE,
};

impl App {
    pub(crate) fn reload_search_tracks_from_db(&mut self) {
        let Some(db) = &self.db else {
            self.search_tracks.clear();
            self.search_total_rows = 0;
            return;
        };

        match db.search_tracks_page(
            &self.search_query,
            self.search_category.id,
            self.search_subcategory.id,
            self.search_genre.id,
            self.search_page_start,
            SEARCH_PAGE_SIZE,
        ) {
            Ok((mut tracks, total_rows)) => {
                let last_start = page_start_for_total(total_rows);
                if self.search_page_start > last_start {
                    self.search_page_start = last_start;
                    match db.search_tracks_page(
                        &self.search_query,
                        self.search_category.id,
                        self.search_subcategory.id,
                        self.search_genre.id,
                        self.search_page_start,
                        SEARCH_PAGE_SIZE,
                    ) {
                        Ok((last_page_tracks, _)) => tracks = last_page_tracks,
                        Err(error) => {
                            self.status = format!("Search reload failed: {error}");
                            return;
                        }
                    }
                }
                self.search_tracks = tracks;
                self.search_total_rows = total_rows;
                if self.selected_search_track_id.is_some_and(|selected_id| {
                    !self
                        .search_tracks
                        .iter()
                        .any(|track| track.id == selected_id)
                }) {
                    self.selected_search_track_id = None;
                }
            }
            Err(error) => self.status = format!("Search reload failed: {error}"),
        }
    }

    pub(crate) fn reload_picker_tracks_from_db(&mut self, window_id: window::Id) {
        let Some(db) = self.db.clone() else {
            if let Some(picker) = self.track_picker_mut(window_id) {
                picker.tracks.clear();
                picker.total_rows = 0;
            }
            return;
        };

        let Some(picker) = self.track_picker(window_id) else {
            return;
        };
        let query = picker.search_query.clone();
        let category_id = picker.search_category.id;
        let subcategory_id = picker.search_subcategory.id;
        let genre_id = picker.search_genre.id;
        let page_start = picker.page_start;

        match db.search_tracks_page(
            &query,
            category_id,
            subcategory_id,
            genre_id,
            page_start,
            SEARCH_PAGE_SIZE,
        ) {
            Ok((mut tracks, total_rows)) => {
                let last_start = page_start_for_total(total_rows);
                let effective_page_start = page_start.min(last_start);
                if page_start > last_start {
                    match db.search_tracks_page(
                        &query,
                        category_id,
                        subcategory_id,
                        genre_id,
                        effective_page_start,
                        SEARCH_PAGE_SIZE,
                    ) {
                        Ok((last_page_tracks, _)) => tracks = last_page_tracks,
                        Err(error) => {
                            self.status = format!("Picker reload failed: {error}");
                            return;
                        }
                    }
                }
                if let Some(picker) = self.track_picker_mut(window_id) {
                    picker.tracks = tracks;
                    picker.total_rows = total_rows;
                    picker.page_start = effective_page_start;
                    if picker.selected_track_id.is_some_and(|selected_id| {
                        !picker.tracks.iter().any(|track| track.id == selected_id)
                    }) {
                        picker.selected_track_id = None;
                    }
                }
            }
            Err(error) => self.status = format!("Picker reload failed: {error}"),
        }
    }

    pub(crate) fn search_track_path(&self, track_id: i32) -> Option<PathBuf> {
        let track = self.search_track(track_id)?;
        let path = track.path.trim();

        (!path.is_empty()).then(|| PathBuf::from(path))
    }

    pub(crate) fn search_track_cue_in(&self, track_id: i32) -> std::time::Duration {
        self.search_track(track_id)
            .map(|track| track.cue_in)
            .unwrap_or_default()
    }

    pub(crate) fn loaded_track(&self, track_id: i32) -> Option<LoadedTrack> {
        let track = self.search_track(track_id)?;
        Some(LoadedTrack {
            id: track.id,
            artist: track.artist_name.clone(),
            title: track.title.clone(),
            duration: track.duration,
            cue_in: track.cue_in,
            path: PathBuf::from(track.path.trim()),
        })
    }

    pub(crate) fn search_track(&self, track_id: i32) -> Option<db::SearchTrack> {
        self.search_tracks
            .iter()
            .find(|track| track.id == track_id)
            .cloned()
            .or_else(|| {
                self.windows.values().find_map(|window| match window {
                    WindowKind::TrackPicker(picker) => picker
                        .tracks
                        .iter()
                        .find(|track| track.id == track_id)
                        .cloned(),
                    WindowKind::Main => None,
                })
            })
            .or_else(|| {
                self.db
                    .as_ref()
                    .and_then(|db| db.search_track(track_id).ok().flatten())
            })
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

    pub(crate) fn last_search_page_start(&self) -> usize {
        page_start_for_total(self.search_total_rows)
    }

    pub(crate) fn last_picker_page_start(&self, picker: &TrackPickerState) -> usize {
        page_start_for_total(picker.total_rows)
    }
}
