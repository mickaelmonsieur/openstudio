use std::collections::HashMap;

use crate::{
    audio, auto_mix_trigger, db, ActiveQueuePlayLog, App, PreloadedQueueEntry, PREVIEW_PLAYER_ID,
    QUEUE_PLAYER_IDS,
};

impl App {
    pub(crate) fn reload_queue_entries_from_db(&mut self) {
        let Some(db) = &self.db else {
            return;
        };

        match db.queue_entries(&self.app_config.timezone) {
            Ok(entries) => {
                self.queue_entries = entries;
                self.selected_queue_index = None;
                self.clear_preloaded_queue_status();
            }
            Err(error) => self.status = format!("Queue reload failed: {error}"),
        }
    }

    pub(crate) fn refill_queue_if_needed(&mut self) {
        if self.queue_entries.len() < 50 {
            self.reload_queue_entries_from_db();
        }
    }

    pub(crate) fn set_auto_mix_status(&mut self, status: impl Into<String>) {
        let status = status.into();
        self.auto_mix_status = status.clone();
        self.log_auto_mix_status(&status);
    }

    pub(crate) fn log_auto_mix_status(&mut self, status: &str) {
        let Some(db) = &self.db else {
            return;
        };

        if let Err(error) = db.insert_automix_log(status) {
            self.status = format!("AUTO MIX log insert failed: {error}");
        }
    }

    pub(crate) fn active_queue_play_log_positions(
        &self,
    ) -> HashMap<audio::PlayerId, std::time::Duration> {
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

    pub(crate) fn begin_queue_play_log(
        &mut self,
        player_id: audio::PlayerId,
        entry: &db::QueueEntry,
    ) {
        let Some(track_id) = entry.track_id else {
            return;
        };

        if let Some(db) = &self.db {
            if let Err(error) = db.mark_track_played(track_id) {
                self.status = format!("Last played update failed: {error}");
            }
        }

        self.active_queue_play_logs.insert(
            player_id,
            ActiveQueuePlayLog {
                track_id,
                cue_in: entry.cue_in,
                cue_out: entry.cue_out,
                duration: entry.duration,
            },
        );
    }

    pub(crate) fn close_finished_queue_play_logs(
        &mut self,
        positions: &HashMap<audio::PlayerId, std::time::Duration>,
    ) {
        for (&player_id, &position) in positions {
            if !self.audio.player(player_id).is_active() {
                self.close_queue_play_log(player_id, position);
            }
        }
    }

    pub(crate) fn close_queue_play_log(
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
        let audible_played_duration = active_log.audible_played_duration(played_duration);
        let count_commercial_campaign = active_log.was_read_to_end(audible_played_duration);

        if let Err(error) = db.insert_play_log(
            active_log.track_id,
            audible_played_duration,
            count_commercial_campaign,
        ) {
            self.status = format!("Play log insert failed: {error}");
        }
    }

    pub(crate) fn load_next_from_queue(&mut self, player_id: audio::PlayerId) {
        if self.play_preloaded_queue_entry(player_id) {
            self.fade_out_previous_queue_players(player_id);
            return;
        }

        if self.queue_entries.is_empty() {
            return;
        }
        let entry = self.queue_entries.remove(0);
        self.adjust_selected_queue_index_after_remove(0);
        self.play_queue_entry(player_id, entry);
        self.fade_out_previous_queue_players(player_id);
    }

    pub(crate) fn apply_startup_playback_config(&mut self) {
        if self.app_config.auto_play_on_start && !self.queue_entries.is_empty() {
            self.play_queue_entry_now(0);
        }
    }

    pub(crate) fn play_queue_entry(&mut self, player_id: audio::PlayerId, entry: db::QueueEntry) {
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
                self.audio.handle(
                    player_id,
                    audio::PlayerCommand::Load {
                        path,
                        cue_in: entry.cue_in,
                    },
                );
                self.audio.handle(player_id, audio::PlayerCommand::Play);
                self.begin_queue_play_log(player_id, &entry);
            }
        }

        self.finalize_queue_entry_launch(player_id, entry);
    }

    pub(crate) fn queue_entry_label(entry: &db::QueueEntry) -> String {
        match (entry.artist_name.trim(), entry.title.trim()) {
            ("", "") => format!("Queue item {}", entry.id),
            ("", title) => title.to_string(),
            (artist, "") => artist.to_string(),
            (artist, title) => format!("{artist} - {title}"),
        }
    }

    pub(crate) fn clear_preloaded_queue_status(&mut self) {
        if self.preloaded_queue_entry.take().is_some() && self.autodj_enabled {
            self.set_auto_mix_status("Waiting");
        }
    }

    pub(crate) fn preload_next_queue_entry(&mut self, player_id: audio::PlayerId) {
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

        self.audio.handle(
            player_id,
            audio::PlayerCommand::Load {
                path,
                cue_in: entry.cue_in,
            },
        );
        self.set_auto_mix_status(format!(
            "Track {} has been preloaded.",
            Self::queue_entry_label(&entry)
        ));
        self.preloaded_queue_entry = Some(PreloadedQueueEntry { player_id, entry });
    }

    pub(crate) fn play_preloaded_queue_entry(&mut self, player_id: audio::PlayerId) -> bool {
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
        self.begin_queue_play_log(player_id, &preloaded.entry);
        self.set_auto_mix_status(format!(
            "Track {} has started.",
            Self::queue_entry_label(&preloaded.entry)
        ));
        self.finalize_queue_entry_launch(player_id, preloaded.entry);
        self.preloaded_queue_entry = None;
        true
    }

    pub(crate) fn finalize_queue_entry_launch(
        &mut self,
        player_id: audio::PlayerId,
        entry: db::QueueEntry,
    ) {
        if self.previewing_queue_id == Some(entry.id) {
            self.stop_preview();
            self.previewing_queue_id = None;
        }

        if let Some(db) = &self.db {
            if let Err(e) = db.delete_queue_entry(entry.id) {
                self.status = format!("Queue entry delete failed: {e}");
            }
        }

        self.refill_queue_if_needed();
        self.queue_player_entries.insert(player_id, entry.clone());
        self.current_queue_player_id = player_id;
        self.current_queue_entry = Some(entry);
        self.update_track_end_at();
    }

    pub(crate) fn play_queue_entry_now(&mut self, index: usize) {
        if index >= self.queue_entries.len() {
            return;
        }

        let player_id = self.queue_player_id_for_immediate_launch();
        let entry = self.queue_entries.remove(index);
        self.adjust_selected_queue_index_after_remove(index);
        self.play_queue_entry(player_id, entry);
        self.fade_out_previous_queue_players(player_id);
    }

    pub(crate) fn any_queue_active(&self) -> bool {
        QUEUE_PLAYER_IDS
            .iter()
            .any(|&player_id| self.audio.player(player_id).is_active())
    }

    pub(crate) fn queue_active_flags(&self) -> [bool; 2] {
        QUEUE_PLAYER_IDS.map(|player_id| self.audio.player(player_id).is_active())
    }

    pub(crate) fn next_queue_player_id(&self, current: audio::PlayerId) -> audio::PlayerId {
        match current {
            audio::PlayerId::QueueA => audio::PlayerId::QueueB,
            audio::PlayerId::QueueB => audio::PlayerId::QueueA,
            _ => audio::PlayerId::QueueA,
        }
    }

    pub(crate) fn queue_player_id_for_immediate_launch(&self) -> audio::PlayerId {
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

    pub(crate) fn configured_fade_out_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.app_config.fade_out_duration_ms.max(0) as u64)
    }

    pub(crate) fn fade_out_previous_queue_players(&mut self, keep_player_id: audio::PlayerId) {
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

    pub(crate) fn adjust_selected_queue_index_after_remove(&mut self, removed_index: usize) {
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

    pub(crate) fn stop_queue_players(&mut self) {
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
        self.update_track_end_at();
        self.current_queue_player_id = audio::PlayerId::QueueA;
    }

    pub(crate) fn sync_queue_players(&mut self, was_active: [bool; 2]) {
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
        self.update_track_end_at();
    }

    pub(crate) fn sync_auto_mix(&mut self) {
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

    pub(crate) fn update_track_end_at(&mut self) {
        let elapsed = self.elapsed();
        let new_end = self
            .current_queue_entry
            .as_ref()
            .map(|e| std::time::SystemTime::now() + e.cue_out.saturating_sub(elapsed));
        let should_update = match (self.track_end_at, new_end) {
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
            (Some(stored), Some(new)) => {
                let diff = if new > stored {
                    new.duration_since(stored).unwrap_or_default()
                } else {
                    stored.duration_since(new).unwrap_or_default()
                };
                diff > std::time::Duration::from_millis(1500)
            }
        };
        if should_update {
            self.track_end_at = new_end;
        }
    }

    pub(crate) fn refresh_current_queue_entry(&mut self) {
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

    pub(crate) fn stop_preview(&mut self) {
        self.audio
            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Stop);
    }

    pub(crate) fn set_auto_dj_enabled(&mut self, enabled: bool) {
        if self.autodj_enabled == enabled {
            return;
        }
        self.autodj_enabled = enabled;
        if self.autodj_enabled {
            self.set_auto_mix_status("Waiting");
        } else {
            self.preloaded_queue_entry = None;
            self.set_auto_mix_status("Disabled");
        }
    }

    pub(crate) fn queue_preview_id_at_position(&self, one_based_id: usize) -> Option<i32> {
        self.queue_entries
            .get(one_based_id.checked_sub(1)?)
            .map(|entry| entry.id)
    }

    pub(crate) fn play_queue_preview_at_position(
        &mut self,
        one_based_id: usize,
        toggle: bool,
    ) -> bool {
        let Some(queue_id) = self.queue_preview_id_at_position(one_based_id) else {
            return false;
        };

        if toggle && self.previewing_queue_id == Some(queue_id) {
            self.stop_preview();
            self.previewing_queue_id = None;
            return true;
        }

        let entry = self.queue_entries.iter().find(|entry| entry.id == queue_id);
        let (track_id, cue_in) = entry
            .map(|entry| (entry.track_id, entry.cue_in))
            .unwrap_or((None, std::time::Duration::ZERO));
        let path = track_id.and_then(|track_id| self.search_track_path(track_id));
        let Some(path) = path else {
            return false;
        };

        self.audio.handle(
            PREVIEW_PLAYER_ID,
            audio::PlayerCommand::Load { path, cue_in },
        );
        self.audio
            .handle(PREVIEW_PLAYER_ID, audio::PlayerCommand::Play);
        self.previewing_queue_id = Some(queue_id);
        true
    }

    pub(crate) fn insert_selected_search_into_queue(&mut self) {
        let Some(track_id) = self.selected_search_track_id else {
            return;
        };
        let Some(track) = self.search_track(track_id) else {
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

    pub(crate) fn replace_selected_queue_entry(&mut self) {
        let Some(track_id) = self.selected_search_track_id else {
            return;
        };
        let Some(queue_index) = self.selected_queue_index else {
            return;
        };
        let Some(track) = self.search_track(track_id) else {
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

    pub(crate) fn remove_selected_queue_entry(&mut self) {
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
                self.refill_queue_if_needed();
            }
            Err(e) => self.status = format!("Queue entry delete failed: {e}"),
        }
    }
}
