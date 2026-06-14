use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::Arc;

use libloading::Library;

use super::encoder::{convert_channels, EncodedAudioFormat, StreamingEncoder};
use super::input::AudioFrame;
use super::StreamingConfig;

type HandleAacEncoder = *mut c_void;
type AacEncError = i32;
type Uint = u32;
type Int = i32;

const AACENC_OK: AacEncError = 0;
const AOT_AAC_LC: Uint = 2;
const MODE_1: Uint = 1;
const MODE_2: Uint = 2;
const TT_MP4_ADTS: Uint = 2;

const AACENC_AOT: Uint = 0x0100;
const AACENC_BITRATE: Uint = 0x0101;
const AACENC_SAMPLERATE: Uint = 0x0103;
const AACENC_CHANNELMODE: Uint = 0x0106;
const AACENC_CHANNELORDER: Uint = 0x0107;
const AACENC_AFTERBURNER: Uint = 0x0200;
const AACENC_TRANSMUX: Uint = 0x0300;

const IN_AUDIO_DATA: Int = 0;
const OUT_BITSTREAM_DATA: Int = 3;

type AacEncOpen = unsafe extern "C" fn(*mut HandleAacEncoder, Uint, Uint) -> AacEncError;
type AacEncClose = unsafe extern "C" fn(*mut HandleAacEncoder) -> AacEncError;
type AacEncoderSetParam = unsafe extern "C" fn(HandleAacEncoder, Uint, Uint) -> AacEncError;
type AacEncEncode = unsafe extern "C" fn(
    HandleAacEncoder,
    *const AacEncBufDesc,
    *const AacEncBufDesc,
    *const AacEncInArgs,
    *mut AacEncOutArgs,
) -> AacEncError;
type AacEncInfo = unsafe extern "C" fn(HandleAacEncoder, *mut AacEncInfoStruct) -> AacEncError;

#[repr(C)]
struct AacEncBufDesc {
    num_bufs: Int,
    bufs: *mut *mut c_void,
    buffer_identifiers: *mut Int,
    buf_sizes: *mut Int,
    buf_el_sizes: *mut Int,
}

#[repr(C)]
struct AacEncInArgs {
    num_in_samples: Int,
    num_anc_bytes: Int,
}

#[repr(C)]
#[derive(Default)]
struct AacEncOutArgs {
    num_out_bytes: Int,
    num_in_samples: Int,
    num_anc_bytes: Int,
}

#[repr(C)]
struct AacEncInfoStruct {
    max_out_buf_bytes: Int,
    max_anc_bytes: Int,
    in_buf_fill_level: Int,
    input_channels: Int,
    frame_length: Int,
    encoder_delay: Int,
    conf_size: Int,
    conf_buf: [u8; 64],
}

impl Default for AacEncInfoStruct {
    fn default() -> Self {
        Self {
            max_out_buf_bytes: 0,
            max_anc_bytes: 0,
            in_buf_fill_level: 0,
            input_channels: 0,
            frame_length: 1024,
            encoder_delay: 0,
            conf_size: 0,
            conf_buf: [0; 64],
        }
    }
}

struct FdkSymbols {
    _lib: Library,
    aac_enc_open: AacEncOpen,
    aac_enc_close: AacEncClose,
    aac_encoder_set_param: AacEncoderSetParam,
    aac_enc_encode: AacEncEncode,
    aac_enc_info: AacEncInfo,
}

impl FdkSymbols {
    fn load() -> Result<Self, String> {
        let (lib, path) = load_fdk_library()?;
        unsafe {
            let aac_enc_open = *lib
                .get::<AacEncOpen>(b"aacEncOpen\0")
                .map_err(|error| format!("libfdk-aac missing aacEncOpen: {error}"))?;
            let aac_enc_close = *lib
                .get::<AacEncClose>(b"aacEncClose\0")
                .map_err(|error| format!("libfdk-aac missing aacEncClose: {error}"))?;
            let aac_encoder_set_param = *lib
                .get::<AacEncoderSetParam>(b"aacEncoder_SetParam\0")
                .map_err(|error| format!("libfdk-aac missing aacEncoder_SetParam: {error}"))?;
            let aac_enc_encode = *lib
                .get::<AacEncEncode>(b"aacEncEncode\0")
                .map_err(|error| format!("libfdk-aac missing aacEncEncode: {error}"))?;
            let aac_enc_info = *lib
                .get::<AacEncInfo>(b"aacEncInfo\0")
                .map_err(|error| format!("libfdk-aac missing aacEncInfo: {error}"))?;
            eprintln!("Loaded libfdk-aac from {}", path.display());
            Ok(Self {
                _lib: lib,
                aac_enc_open,
                aac_enc_close,
                aac_encoder_set_param,
                aac_enc_encode,
                aac_enc_info,
            })
        }
    }
}

pub fn is_available() -> bool {
    load_fdk_library().is_ok()
}

fn load_fdk_library() -> Result<(Library, PathBuf), String> {
    let mut candidates = library_candidates();
    candidates.dedup();
    let mut last_error = String::new();
    for candidate in candidates {
        match unsafe { Library::new(&candidate) } {
            Ok(lib) => return Ok((lib, candidate)),
            Err(error) => last_error = error.to_string(),
        }
    }
    Err(if last_error.is_empty() {
        String::from("libfdk-aac not found")
    } else {
        format!("libfdk-aac not found ({last_error})")
    })
}

fn library_candidates() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    const NAMES: &[&str] = &["libfdk-aac.dylib", "libfdk-aac.2.dylib"];
    #[cfg(target_os = "linux")]
    const NAMES: &[&str] = &["libfdk-aac.so", "libfdk-aac.so.2"];
    #[cfg(target_os = "windows")]
    const NAMES: &[&str] = &["libfdk-aac-2.dll", "fdk-aac.dll", "libfdk-aac.dll"];
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    const NAMES: &[&str] = &["libfdk-aac.so"];

    let mut candidates = NAMES.iter().map(PathBuf::from).collect::<Vec<_>>();

    #[cfg(target_os = "macos")]
    for dir in ["/opt/homebrew/lib", "/usr/local/lib", "/usr/lib"] {
        for name in NAMES {
            candidates.push(PathBuf::from(dir).join(name));
        }
    }
    #[cfg(target_os = "macos")]
    candidates.push(PathBuf::from(
        "/Library/Application Support/butt/libfdk-aac.2.dylib",
    ));

    #[cfg(target_os = "linux")]
    for dir in [
        "/usr/lib",
        "/usr/local/lib",
        "/usr/lib/x86_64-linux-gnu",
        "/lib/x86_64-linux-gnu",
    ] {
        for name in NAMES {
            candidates.push(PathBuf::from(dir).join(name));
        }
    }

    candidates
}

pub struct FdkAacLcEncoder {
    symbols: Arc<FdkSymbols>,
    handle: HandleAacEncoder,
    channels: usize,
    frame_samples: usize,
}

unsafe impl Send for FdkAacLcEncoder {}

impl FdkAacLcEncoder {
    pub fn new(config: &StreamingConfig) -> Result<Self, String> {
        let symbols = Arc::new(FdkSymbols::load()?);
        let channels = config.channels.clamp(1, 2) as usize;
        let channel_mode = if channels == 1 { MODE_1 } else { MODE_2 };
        let mut handle: HandleAacEncoder = std::ptr::null_mut();
        let open_result = unsafe { (symbols.aac_enc_open)(&mut handle, 0, channels as Uint) };
        if open_result != AACENC_OK || handle.is_null() {
            return Err(format!("libfdk-aac open failed with code {open_result}"));
        }

        let mut encoder = Self {
            symbols,
            handle,
            channels,
            frame_samples: 1024,
        };

        encoder.set_param(AACENC_AOT, AOT_AAC_LC)?;
        encoder.set_param(
            AACENC_SAMPLERATE,
            config.sample_rate.clamp(8_000, 48_000) as Uint,
        )?;
        encoder.set_param(AACENC_CHANNELMODE, channel_mode)?;
        encoder.set_param(AACENC_CHANNELORDER, 1)?;
        encoder.set_param(
            AACENC_BITRATE,
            config.bitrate_kbps.clamp(8, 320) as Uint * 1000,
        )?;
        encoder.set_param(AACENC_TRANSMUX, TT_MP4_ADTS)?;
        encoder.set_param(AACENC_AFTERBURNER, 1)?;
        encoder.init()?;
        Ok(encoder)
    }

    fn set_param(&mut self, param: Uint, value: Uint) -> Result<(), String> {
        let result = unsafe { (self.symbols.aac_encoder_set_param)(self.handle, param, value) };
        if result == AACENC_OK {
            Ok(())
        } else {
            Err(format!(
                "libfdk-aac set param 0x{param:04x}={value} failed with code {result}"
            ))
        }
    }

    fn init(&mut self) -> Result<(), String> {
        let result = unsafe {
            (self.symbols.aac_enc_encode)(
                self.handle,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        };
        if result != AACENC_OK {
            return Err(format!("libfdk-aac init failed with code {result}"));
        }

        let mut info = AacEncInfoStruct::default();
        let info_result = unsafe { (self.symbols.aac_enc_info)(self.handle, &mut info) };
        if info_result == AACENC_OK && info.frame_length > 0 {
            self.frame_samples = info.frame_length as usize;
        }
        Ok(())
    }

    fn encode_aac(&mut self, frame: &AudioFrame) -> Result<Vec<u8>, String> {
        let pcm = convert_channels(&frame.samples, frame.channels, self.channels)
            .into_iter()
            .map(f32_to_i32_pcm)
            .collect::<Vec<_>>();
        if pcm.is_empty() {
            return Ok(Vec::new());
        }

        let mut out = vec![0u8; 8192];
        let mut in_ptr = pcm.as_ptr() as *mut c_void;
        let mut out_ptr = out.as_mut_ptr() as *mut c_void;
        let mut in_id = IN_AUDIO_DATA;
        let mut out_id = OUT_BITSTREAM_DATA;
        let mut in_size = (pcm.len() * std::mem::size_of::<i32>()) as Int;
        let mut out_size = out.len() as Int;
        let mut in_el_size = std::mem::size_of::<i32>() as Int;
        let mut out_el_size = std::mem::size_of::<u8>() as Int;

        let in_desc = AacEncBufDesc {
            num_bufs: 1,
            bufs: &mut in_ptr,
            buffer_identifiers: &mut in_id,
            buf_sizes: &mut in_size,
            buf_el_sizes: &mut in_el_size,
        };
        let out_desc = AacEncBufDesc {
            num_bufs: 1,
            bufs: &mut out_ptr,
            buffer_identifiers: &mut out_id,
            buf_sizes: &mut out_size,
            buf_el_sizes: &mut out_el_size,
        };
        let in_args = AacEncInArgs {
            num_in_samples: pcm.len() as Int,
            num_anc_bytes: 0,
        };
        let mut out_args = AacEncOutArgs::default();

        let result = unsafe {
            (self.symbols.aac_enc_encode)(self.handle, &in_desc, &out_desc, &in_args, &mut out_args)
        };
        if result != AACENC_OK {
            return Err(format!("libfdk-aac encode failed with code {result}"));
        }

        out.truncate(out_args.num_out_bytes.max(0) as usize);
        Ok(out)
    }
}

fn f32_to_i32_pcm(sample: f32) -> i32 {
    let scaled = sample.clamp(-1.0, 1.0) as f64 * i32::MAX as f64;
    scaled.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32
}

impl StreamingEncoder for FdkAacLcEncoder {
    fn format(&self) -> EncodedAudioFormat {
        EncodedAudioFormat::Aac
    }

    fn frame_samples(&self) -> usize {
        self.frame_samples
    }

    fn encode(&mut self, frame: &AudioFrame) -> Result<Vec<u8>, String> {
        self.encode_aac(frame)
    }
}

impl Drop for FdkAacLcEncoder {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                let _ = (self.symbols.aac_enc_close)(&mut self.handle);
            }
        }
    }
}
