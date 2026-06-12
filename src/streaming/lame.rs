use std::ffi::c_void;
use std::os::raw::{c_int, c_uchar};

use crate::audio::BroadcastFrame;

use super::StreamingConfig;

type LameGlobalFlags = c_void;

extern "C" {
    fn lame_init() -> *mut LameGlobalFlags;
    fn lame_close(gfp: *mut LameGlobalFlags) -> c_int;
    fn lame_set_in_samplerate(gfp: *mut LameGlobalFlags, sample_rate: c_int) -> c_int;
    fn lame_set_out_samplerate(gfp: *mut LameGlobalFlags, sample_rate: c_int) -> c_int;
    fn lame_set_num_channels(gfp: *mut LameGlobalFlags, channels: c_int) -> c_int;
    fn lame_set_brate(gfp: *mut LameGlobalFlags, bitrate: c_int) -> c_int;
    fn lame_set_quality(gfp: *mut LameGlobalFlags, quality: c_int) -> c_int;
    fn lame_init_params(gfp: *mut LameGlobalFlags) -> c_int;
    fn lame_encode_buffer_interleaved_ieee_float(
        gfp: *mut LameGlobalFlags,
        pcm: *const f32,
        num_samples: c_int,
        mp3buf: *mut c_uchar,
        mp3buf_size: c_int,
    ) -> c_int;
    fn lame_encode_flush(gfp: *mut LameGlobalFlags, mp3buf: *mut c_uchar, size: c_int) -> c_int;
}

pub struct LameMp3Encoder {
    gfp: *mut LameGlobalFlags,
    channels: usize,
}

impl LameMp3Encoder {
    pub fn new(config: &StreamingConfig) -> Result<Self, String> {
        let gfp = unsafe { lame_init() };
        if gfp.is_null() {
            return Err(String::from("lame_init returned null"));
        }

        let channels = config.channels.clamp(1, 2) as usize;
        let result = unsafe {
            lame_set_in_samplerate(gfp, config.sample_rate)
                | lame_set_out_samplerate(gfp, config.sample_rate)
                | lame_set_num_channels(gfp, channels as c_int)
                | lame_set_brate(gfp, config.bitrate_kbps)
                | lame_set_quality(gfp, 2)
                | lame_init_params(gfp)
        };
        if result < 0 {
            unsafe {
                lame_close(gfp);
            }
            return Err(format!("LAME init failed with code {result}"));
        }

        Ok(Self { gfp, channels })
    }

    pub fn encode(&mut self, frame: &BroadcastFrame) -> Result<Vec<u8>, String> {
        let pcm = convert_channels(&frame.samples, frame.channels, self.channels);
        let frames = pcm.len() / self.channels.max(1);
        if frames == 0 {
            return Ok(Vec::new());
        }

        let mp3_capacity = (1.25 * frames as f32) as usize + 7200;
        let mut mp3 = vec![0u8; mp3_capacity];
        let written = unsafe {
            lame_encode_buffer_interleaved_ieee_float(
                self.gfp,
                pcm.as_ptr(),
                frames as c_int,
                mp3.as_mut_ptr(),
                mp3.len() as c_int,
            )
        };
        if written < 0 {
            return Err(format!("lame_encode failed with code {written}"));
        }
        mp3.truncate(written as usize);
        Ok(mp3)
    }
}

impl Drop for LameMp3Encoder {
    fn drop(&mut self) {
        let mut buffer = vec![0u8; 7200];
        unsafe {
            let _ = lame_encode_flush(self.gfp, buffer.as_mut_ptr(), buffer.len() as c_int);
            let _ = lame_close(self.gfp);
        }
    }
}

fn convert_channels(samples: &[f32], from_channels: usize, to_channels: usize) -> Vec<f32> {
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
