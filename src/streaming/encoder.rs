use super::input::AudioFrame;
use super::StreamingConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodedAudioFormat {
    Mp3,
    Aac,
}

pub trait StreamingEncoder: Send {
    fn format(&self) -> EncodedAudioFormat;
    fn frame_samples(&self) -> usize;
    fn encode(&mut self, frame: &AudioFrame) -> Result<Vec<u8>, String>;
}

pub fn encoder_type_from_config(config: &StreamingConfig) -> &'static str {
    match config.encoder_type.trim().to_ascii_lowercase().as_str() {
        "aac-lc" => "aac-lc",
        _ => "mp3",
    }
}

#[cfg(all(openstudio_has_lame, openstudio_has_shout))]
pub fn new_encoder(config: &StreamingConfig) -> Result<Box<dyn StreamingEncoder>, String> {
    match encoder_type_from_config(config) {
        "aac-lc" => Ok(Box::new(super::fdk_aac::FdkAacLcEncoder::new(config)?)),
        _ => Ok(Box::new(super::lame::LameMp3Encoder::new(config)?)),
    }
}

pub fn convert_channels(samples: &[f32], from_channels: usize, to_channels: usize) -> Vec<f32> {
    match (from_channels, to_channels) {
        (2, 2) => samples.to_vec(),
        (2, 1) => samples
            .chunks_exact(2)
            .map(|frame| (frame[0] + frame[1]) * 0.5)
            .collect(),
        (1, 2) => samples
            .iter()
            .flat_map(|&sample| [sample, sample])
            .collect(),
        (1, 1) => samples.to_vec(),
        (_, 1) => samples
            .chunks(from_channels.max(1))
            .map(|frame| frame.iter().copied().sum::<f32>() / frame.len().max(1) as f32)
            .collect(),
        (_, 2) => samples
            .chunks(from_channels.max(1))
            .flat_map(|frame| {
                let left = frame.first().copied().unwrap_or_default();
                let right = frame.get(1).copied().unwrap_or(left);
                [left, right]
            })
            .collect(),
        _ => Vec::new(),
    }
}
