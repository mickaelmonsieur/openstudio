use crate::{db, streaming, App};

impl App {
    pub(crate) fn sync_streaming_encoder(&mut self) {
        if !self.app_config.encoder_enabled || self.app_config.encoder_password.trim().is_empty() {
            self.stop_streaming_encoder();
            return;
        }

        let config = streaming::StreamingConfig::from(&self.app_config);
        if self.streaming_handle.is_some() {
            self.stop_streaming_encoder();
        }

        self.streaming_handle = Some(streaming::start(config));
        self.sync_streaming_metadata_for_current_track();
        self.ensure_streaming_metadata_fallback();
    }

    pub(crate) fn stop_streaming_encoder(&mut self) {
        if let Some(handle) = self.streaming_handle.take() {
            handle.stop();
        }
    }

    pub(crate) fn streaming_status(&self) -> String {
        if !self.app_config.encoder_enabled {
            return String::from("Disabled");
        }
        if self.app_config.encoder_password.trim().is_empty() {
            return String::from("Not configured: password required");
        }
        self.streaming_handle
            .as_ref()
            .map(|handle| handle.status())
            .unwrap_or_else(|| String::from("Stopped"))
    }

    pub(crate) fn streaming_diagnostics(&self) -> streaming::InputDiagnostics {
        self.streaming_handle
            .as_ref()
            .map(|handle| handle.input_diagnostics())
            .unwrap_or_default()
    }

    pub(crate) fn streaming_kbps(&self) -> u64 {
        self.streaming_handle
            .as_ref()
            .map(|handle| handle.kbps())
            .unwrap_or(0)
    }

    pub(crate) fn streaming_timing(&self) -> streaming::StreamingTiming {
        self.streaming_handle
            .as_ref()
            .map(|handle| handle.timing())
            .unwrap_or_else(|| streaming::StreamingTiming {
                launched_at: String::from("-"),
                uptime: String::from("00:00:00"),
                last_reconnect_at: String::from("-"),
            })
    }

    pub(crate) fn sync_streaming_metadata_for_current_track(&self) {
        let Some(entry) = self.current_queue_entry.as_ref() else {
            self.ensure_streaming_metadata_fallback();
            return;
        };
        self.sync_streaming_metadata_for_entry(entry);
    }

    pub(crate) fn sync_streaming_metadata_for_entry(&self, entry: &db::QueueEntry) {
        let Some(handle) = self.streaming_handle.as_ref() else {
            return;
        };
        if entry.track_type_name.eq_ignore_ascii_case("Music") {
            handle.set_title_metadata(queue_entry_label(entry));
        } else {
            handle.set_title_metadata(self.streaming_metadata_fallback());
        }
    }

    fn ensure_streaming_metadata_fallback(&self) {
        let Some(handle) = self.streaming_handle.as_ref() else {
            return;
        };
        let has_music_metadata = self
            .current_queue_entry
            .as_ref()
            .is_some_and(|entry| entry.track_type_name.eq_ignore_ascii_case("Music"));
        if !has_music_metadata {
            handle.set_title_metadata(self.streaming_metadata_fallback());
        }
    }

    fn streaming_metadata_fallback(&self) -> String {
        let name = self.station_name.trim();
        if name.is_empty() {
            String::from("OpenStudio")
        } else {
            name.to_string()
        }
    }
}

fn queue_entry_label(entry: &db::QueueEntry) -> String {
    match (entry.artist_name.trim(), entry.title.trim()) {
        ("", "") => format!("Queue item {}", entry.id),
        ("", title) => title.to_string(),
        (artist, "") => artist.to_string(),
        (artist, title) => format!("{artist} - {title}"),
    }
}
