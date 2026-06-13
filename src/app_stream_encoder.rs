use iced::Task;

use crate::{db, App, Dialog, Message};

pub(crate) const ENCODER_FIXED_SERVER_HOST: &str = "openstudio.entrypoint.belstream.net";
pub(crate) const ENCODER_FIXED_SERVER_PORT: i32 = 80;
pub(crate) const ENCODER_TYPE_MP3: &str = "mp3";
pub(crate) const ENCODER_TYPE_MP3_LABEL: &str = "MPEG-1/2 Audio Layer III (LAME)";

impl App {
    pub(crate) fn open_stream_encoder_dialog(&mut self) {
        self.dialog = Some(Dialog::StreamEncoder {
            enabled: self.app_config.encoder_enabled,
            bitrate: self.app_config.encoder_bitrate.clamp(8, 320).to_string(),
            sample_rate: self
                .app_config
                .encoder_sample_rate
                .clamp(8000, 48000)
                .to_string(),
            channels: channels_label(self.app_config.encoder_channels).into(),
            encoder_type: ENCODER_TYPE_MP3_LABEL.into(),
            server_host: ENCODER_FIXED_SERVER_HOST.into(),
            server_port: ENCODER_FIXED_SERVER_PORT.to_string(),
            password: self.app_config.encoder_password.clone(),
            mountpoint: self.app_config.encoder_mountpoint.clone(),
            reconnect_seconds: self.app_config.encoder_reconnect_seconds.to_string(),
            error: None,
        });
    }

    pub(crate) fn update_stream_encoder_dialog(
        &mut self,
        update: impl FnOnce(&mut Dialog),
    ) -> Task<Message> {
        if let Some(dialog) = &mut self.dialog {
            update(dialog);
        }
        Task::none()
    }

    pub(crate) fn set_stream_encoder_bitrate_input(&mut self, value: String) -> Task<Message> {
        let value = bounded_numeric_input(value, 320);
        self.update_stream_encoder_dialog(|dialog| {
            if let Dialog::StreamEncoder { bitrate, .. } = dialog {
                *bitrate = value;
            }
        })
    }

    pub(crate) fn set_stream_encoder_sample_rate_input(&mut self, value: String) -> Task<Message> {
        let value = bounded_numeric_input(value, 48000);
        self.update_stream_encoder_dialog(|dialog| {
            if let Dialog::StreamEncoder { sample_rate, .. } = dialog {
                *sample_rate = value;
            }
        })
    }

    pub(crate) fn set_stream_encoder_channels_input(&mut self, value: String) -> Task<Message> {
        let value = if value == "Mono" || value == "Stéréo" {
            value
        } else {
            String::from("Stéréo")
        };
        self.update_stream_encoder_dialog(|dialog| {
            if let Dialog::StreamEncoder { channels, .. } = dialog {
                *channels = value;
            }
        })
    }

    pub(crate) fn set_stream_encoder_mountpoint_input(&mut self, value: String) -> Task<Message> {
        let value = value
            .chars()
            .filter(|ch| !ch.is_control() && !ch.is_whitespace())
            .take(128)
            .collect::<String>();
        self.update_stream_encoder_dialog(|dialog| {
            if let Dialog::StreamEncoder { mountpoint, .. } = dialog {
                *mountpoint = value;
            }
        })
    }

    pub(crate) fn set_stream_encoder_reconnect_input(&mut self, value: String) -> Task<Message> {
        let value = bounded_numeric_input(value, 3600);
        self.update_stream_encoder_dialog(|dialog| {
            if let Dialog::StreamEncoder {
                reconnect_seconds, ..
            } = dialog
            {
                *reconnect_seconds = value;
            }
        })
    }

    pub(crate) fn stream_encoder_password_is_valid(&self) -> bool {
        let Some(Dialog::StreamEncoder { password, .. }) = &self.dialog else {
            return false;
        };
        password
            .chars()
            .any(|ch| !ch.is_control() && !ch.is_whitespace())
    }

    pub(crate) fn stream_encoder_enabled_in_dialog(&self) -> bool {
        matches!(
            &self.dialog,
            Some(Dialog::StreamEncoder { enabled: true, .. })
        )
    }

    pub(crate) fn set_stream_encoder_error(&mut self, message: impl Into<String>) {
        if let Some(Dialog::StreamEncoder { error, .. }) = &mut self.dialog {
            *error = Some(message.into());
        }
    }

    pub(crate) fn stream_encoder_config_from_dialog(&self) -> Option<db::AppConfig> {
        let Some(Dialog::StreamEncoder {
            enabled,
            bitrate,
            sample_rate,
            channels,
            password,
            mountpoint,
            reconnect_seconds,
            ..
        }) = &self.dialog
        else {
            return None;
        };

        let mut cfg = self.app_config.clone();
        cfg.encoder_enabled = *enabled;
        cfg.encoder_bitrate = parse_i32_or_current(bitrate, cfg.encoder_bitrate).clamp(8, 320);
        cfg.encoder_sample_rate =
            parse_i32_or_current(sample_rate, cfg.encoder_sample_rate).clamp(8000, 48000);
        cfg.encoder_channels = channels_value(channels);
        cfg.encoder_type = String::from(ENCODER_TYPE_MP3);
        cfg.encoder_server_host = ENCODER_FIXED_SERVER_HOST.into();
        cfg.encoder_server_port = ENCODER_FIXED_SERVER_PORT.clamp(1, 65535);
        cfg.encoder_password = password.chars().filter(|ch| !ch.is_control()).collect();
        cfg.encoder_mountpoint = normalized_mountpoint(mountpoint);
        cfg.encoder_reconnect_seconds =
            parse_i32_or_current(reconnect_seconds, cfg.encoder_reconnect_seconds).clamp(1, 3600);
        Some(cfg)
    }
}

fn parse_i32_or_current(value: &str, current: i32) -> i32 {
    value.trim().parse::<i32>().unwrap_or(current)
}

fn channels_label(value: i32) -> &'static str {
    if value == 1 {
        "Mono"
    } else {
        "Stéréo"
    }
}

fn channels_value(value: &str) -> i32 {
    if value == "Mono" {
        1
    } else {
        2
    }
}

fn bounded_numeric_input(value: String, max: i32) -> String {
    let digits = value
        .chars()
        .filter(char::is_ascii_digit)
        .take(6)
        .collect::<String>();
    if digits.is_empty() {
        return digits;
    }
    digits
        .parse::<i32>()
        .map(|parsed| parsed.min(max).to_string())
        .unwrap_or_default()
}

fn normalized_mountpoint(value: &str) -> String {
    let trimmed = value.trim();
    let cleaned = trimmed
        .chars()
        .filter(|ch| !ch.is_control() && !ch.is_whitespace())
        .take(128)
        .collect::<String>();
    if cleaned.is_empty() {
        String::from("/live")
    } else if cleaned.starts_with('/') {
        cleaned
    } else {
        format!("/{cleaned}")
    }
}
