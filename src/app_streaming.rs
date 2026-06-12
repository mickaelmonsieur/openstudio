use crate::{db, streaming, App};

impl App {
    pub(crate) fn sync_streaming_encoder(&mut self) {
        if !self.app_config.encoder_enabled || self.app_config.encoder_password.trim().is_empty() {
            self.stop_streaming_encoder();
            return;
        }

        let config = streaming::StreamingConfig::from(&self.app_config);
        if let Some(handle) = &self.streaming_handle {
            handle.update_config(config);
            return;
        }

        let Some(rx) = self.audio.take_broadcast_output() else {
            self.status = String::from("Streaming unavailable (broadcast bus already attached)");
            return;
        };

        self.streaming_handle = Some(streaming::start(config, rx));
        self.sync_streaming_metadata_for_current_track();
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

    pub(crate) fn sync_streaming_metadata_for_current_track(&self) {
        let Some(entry) = self.current_queue_entry.as_ref() else {
            return;
        };
        self.sync_streaming_metadata_for_entry(entry);
    }

    pub(crate) fn sync_streaming_metadata_for_entry(&self, entry: &db::QueueEntry) {
        if !entry.track_type_name.eq_ignore_ascii_case("Music") {
            return;
        }
        let Some(handle) = self.streaming_handle.as_ref() else {
            return;
        };
        handle.set_title_metadata(queue_entry_label(entry));
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
