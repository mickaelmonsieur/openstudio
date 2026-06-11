use crate::{audio, db, App, Dialog};

#[derive(Debug, Clone, Copy)]
pub(crate) struct CompressorPreset {
    pub(crate) name: &'static str,
    attack_ms: f32,
    ratio: f32,
    threshold_db: f32,
    gain_db: f32,
    release_ms: f32,
}

pub(crate) const COMPRESSOR_PRESETS: [CompressorPreset; 5] = [
    CompressorPreset {
        name: "Soft 1",
        attack_ms: 35.0,
        ratio: 1.8,
        threshold_db: -18.0,
        gain_db: 1.0,
        release_ms: 700.0,
    },
    CompressorPreset {
        name: "Soft 2",
        attack_ms: 25.0,
        ratio: 2.5,
        threshold_db: -22.0,
        gain_db: 2.0,
        release_ms: 900.0,
    },
    CompressorPreset {
        name: "Medium",
        attack_ms: 15.0,
        ratio: 3.5,
        threshold_db: -24.0,
        gain_db: 3.0,
        release_ms: 800.0,
    },
    CompressorPreset {
        name: "Strong",
        attack_ms: 8.0,
        ratio: 5.0,
        threshold_db: -28.0,
        gain_db: 4.0,
        release_ms: 650.0,
    },
    CompressorPreset {
        name: "Voice",
        attack_ms: 5.0,
        ratio: 3.0,
        threshold_db: -20.0,
        gain_db: 2.5,
        release_ms: 280.0,
    },
];

pub(crate) fn find_compressor_preset(name: &str) -> Option<CompressorPreset> {
    COMPRESSOR_PRESETS
        .iter()
        .copied()
        .find(|preset| preset.name == name)
}

pub(crate) fn normalized_eq_gains(gains: &[f32]) -> Vec<f32> {
    let mut normalized = gains
        .iter()
        .take(10)
        .map(|gain| gain.clamp(-15.0, 15.0))
        .collect::<Vec<_>>();
    normalized.resize(10, 0.0);
    normalized
}

impl App {
    pub(crate) fn apply_audio_device_config(&mut self, cfg: &db::AppConfig) {
        let to_opt = |s: &str| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        };
        for player_id in [audio::PlayerId::QueueA, audio::PlayerId::QueueB] {
            self.audio
                .player_mut(player_id)
                .set_device(to_opt(&cfg.device_deck));
        }
        self.audio
            .player_mut(audio::PlayerId::Instant)
            .set_device(to_opt(&cfg.device_instant));
        for player_id in [
            audio::PlayerId::Aux1,
            audio::PlayerId::Aux2,
            audio::PlayerId::Aux3,
        ] {
            self.audio
                .player_mut(player_id)
                .set_device(to_opt(&cfg.device_aux));
        }
        self.audio
            .player_mut(audio::PlayerId::Preview)
            .set_device(to_opt(&cfg.device_preview));
    }

    pub(crate) fn apply_audio_processing_config(&mut self, cfg: &db::AppConfig) {
        self.audio
            .set_processing_bypassed(cfg.audio_processing_bypassed);
        self.audio
            .set_master_volume_percent(cfg.audio_master_volume_percent);
        self.audio.set_eq_enabled(cfg.audio_eq_enabled);
        for (index, gain) in normalized_eq_gains(&cfg.audio_eq_gains)
            .into_iter()
            .enumerate()
        {
            self.audio.set_eq_gain_db(index, gain);
        }
        self.audio
            .set_compressor_attack_ms(cfg.audio_compressor_attack_ms);
        self.audio.set_compressor_ratio(cfg.audio_compressor_ratio);
        self.audio
            .set_compressor_threshold_db(cfg.audio_compressor_threshold_db);
        self.audio
            .set_compressor_gain_db(cfg.audio_compressor_gain_db);
        self.audio
            .set_compressor_release_ms(cfg.audio_compressor_release_ms);
        let agc_preset =
            audio::AgcPreset::from_str(&cfg.audio_agc_preset).unwrap_or(audio::AgcPreset::Disabled);
        self.audio.set_agc_preset(agc_preset);
    }

    pub(crate) fn audio_processing_config_from_dialog(&self) -> Option<db::AppConfig> {
        let Some(Dialog::AudioProcessing {
            processing_bypassed,
            input_volume,
            compressor_mode,
            compressor_preset,
            attack,
            ratio,
            threshold,
            gain,
            release,
            eq_enabled,
            eq_gains,
            agc_preset,
        }) = &self.dialog
        else {
            return None;
        };

        let mut cfg = self.app_config.clone();
        cfg.audio_processing_bypassed = *processing_bypassed;
        cfg.audio_master_volume_percent = input_volume.clamp(0.0, 100.0);
        cfg.audio_eq_enabled = *eq_enabled;
        cfg.audio_eq_gains = normalized_eq_gains(eq_gains);
        cfg.audio_compressor_mode = compressor_mode.clone();
        cfg.audio_compressor_preset = compressor_preset.clone();
        cfg.audio_compressor_attack_ms = attack
            .trim()
            .parse::<f32>()
            .unwrap_or(cfg.audio_compressor_attack_ms)
            .clamp(0.1, 5000.0);
        cfg.audio_compressor_ratio = ratio
            .trim()
            .parse::<f32>()
            .unwrap_or(cfg.audio_compressor_ratio)
            .clamp(1.0, 40.0);
        cfg.audio_compressor_threshold_db = threshold
            .trim()
            .parse::<f32>()
            .unwrap_or(cfg.audio_compressor_threshold_db)
            .clamp(-80.0, 0.0);
        cfg.audio_compressor_gain_db = gain
            .trim()
            .parse::<f32>()
            .unwrap_or(cfg.audio_compressor_gain_db)
            .clamp(-24.0, 24.0);
        cfg.audio_compressor_release_ms = release
            .trim()
            .parse::<f32>()
            .unwrap_or(cfg.audio_compressor_release_ms)
            .clamp(1.0, 10000.0);
        cfg.audio_agc_preset = audio::AgcPreset::from_str(agc_preset)
            .unwrap_or(audio::AgcPreset::Disabled)
            .as_str()
            .into();
        Some(cfg)
    }

    pub(crate) fn apply_compressor_preset(&mut self, preset: CompressorPreset) {
        self.audio.set_compressor_attack_ms(preset.attack_ms);
        self.audio.set_compressor_ratio(preset.ratio);
        self.audio.set_compressor_threshold_db(preset.threshold_db);
        self.audio.set_compressor_gain_db(preset.gain_db);
        self.audio.set_compressor_release_ms(preset.release_ms);

        if let Some(Dialog::AudioProcessing {
            compressor_mode,
            attack,
            ratio,
            threshold,
            gain,
            release,
            ..
        }) = &mut self.dialog
        {
            *compressor_mode = "By Preset".into();
            *attack = format!("{:.1}", preset.attack_ms);
            *ratio = format!("{:.2}", preset.ratio);
            *threshold = format!("{:.1}", preset.threshold_db);
            *gain = format!("{:.1}", preset.gain_db);
            *release = format!("{:.1}", preset.release_ms);
        }
    }
}
