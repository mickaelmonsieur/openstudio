use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender, SyncSender, TrySendError};
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, SupportedBufferSize, SupportedStreamConfig};
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

const DEVICE_ID_NAME_PREFIX: &str = "name:";
#[cfg(target_os = "macos")]
const DEVICE_ID_COREAUDIO_PREFIX: &str = "coreaudio:";

const EQ_BAND_FREQS: [f32; 10] = [
    32.0, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerId {
    QueueA,
    QueueB,
    Instant,
    Aux1,
    Aux2,
    Aux3,
    Preview,
}

impl PlayerId {
    pub const ALL: [Self; 7] = [
        Self::QueueA,
        Self::QueueB,
        Self::Instant,
        Self::Aux1,
        Self::Aux2,
        Self::Aux3,
        Self::Preview,
    ];
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Load { path: PathBuf, cue_in: Duration },
    Play,
    Pause,
    Resume,
    TogglePause,
    Stop,
    SoftStop(Duration),
    FadeOut(Duration),
    Restart,
    SeekRelative(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Empty,
    Loaded,
    Playing,
    Paused,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub state: PlayerState,
    pub loaded_path: Option<PathBuf>,
    pub position: Duration,
    pub duration: Option<Duration>,
}

pub struct AudioManager {
    players: HashMap<PlayerId, AudioPlayer>,
    processing: Arc<AudioProcessingSettings>,
}

impl AudioManager {
    pub fn new() -> Self {
        let processing = Arc::new(AudioProcessingSettings::default());
        let players = PlayerId::ALL
            .into_iter()
            .map(|id| (id, AudioPlayer::new(id, Arc::clone(&processing))))
            .collect();

        Self {
            players,
            processing,
        }
    }

    pub fn player(&self, id: PlayerId) -> &AudioPlayer {
        self.players
            .get(&id)
            .expect("audio manager initialized with all players")
    }

    pub fn player_mut(&mut self, id: PlayerId) -> &mut AudioPlayer {
        self.players
            .get_mut(&id)
            .expect("audio manager initialized with all players")
    }

    pub fn handle(&mut self, id: PlayerId, command: PlayerCommand) {
        self.player_mut(id).handle(command);
    }

    pub fn poll(&mut self) {
        for player in self.players.values_mut() {
            player.poll();
        }
    }

    pub fn any_active(&self) -> bool {
        self.players.values().any(AudioPlayer::is_active)
    }

    pub fn set_master_volume_percent(&self, volume: f32) {
        self.processing.set_master_volume_percent(volume);
    }

    pub fn master_volume_percent(&self) -> f32 {
        self.processing.master_volume_percent()
    }

    pub fn set_processing_bypassed(&self, bypassed: bool) {
        self.processing.set_bypassed(bypassed);
    }

    pub fn processing_bypassed(&self) -> bool {
        self.processing.bypassed()
    }

    pub fn set_eq_enabled(&self, enabled: bool) {
        self.processing.set_eq_enabled(enabled);
    }

    pub fn eq_enabled(&self) -> bool {
        self.processing.eq_enabled()
    }

    pub fn set_eq_gain_db(&self, band: usize, gain_db: f32) {
        self.processing.set_eq_gain_db(band, gain_db);
    }

    pub fn eq_gains_db(&self) -> Vec<f32> {
        self.processing.eq_gains_db()
    }

    pub fn set_compressor_attack_ms(&self, value: f32) {
        self.processing.set_compressor_attack_ms(value);
    }

    pub fn compressor_attack_ms(&self) -> f32 {
        self.processing.compressor_attack_ms()
    }

    pub fn set_compressor_ratio(&self, value: f32) {
        self.processing.set_compressor_ratio(value);
    }

    pub fn compressor_ratio(&self) -> f32 {
        self.processing.compressor_ratio()
    }

    pub fn set_compressor_threshold_db(&self, value: f32) {
        self.processing.set_compressor_threshold_db(value);
    }

    pub fn compressor_threshold_db(&self) -> f32 {
        self.processing.compressor_threshold_db()
    }

    pub fn set_compressor_gain_db(&self, value: f32) {
        self.processing.set_compressor_gain_db(value);
    }

    pub fn compressor_gain_db(&self) -> f32 {
        self.processing.compressor_gain_db()
    }

    pub fn set_compressor_release_ms(&self, value: f32) {
        self.processing.set_compressor_release_ms(value);
    }

    pub fn compressor_release_ms(&self) -> f32 {
        self.processing.compressor_release_ms()
    }

    pub fn set_agc_preset(&self, preset: AgcPreset) {
        self.processing.set_agc_preset(preset);
    }

    pub fn agc_preset(&self) -> AgcPreset {
        self.processing.agc_preset()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AudioProcessingSettings {
    bypassed: AtomicBool,
    master_volume_per_mille: AtomicU32,
    agc_preset: AtomicU32,
    eq_enabled: AtomicBool,
    eq_gains_tenth_db: [AtomicI32; 10],
    compressor_attack_tenth_ms: AtomicU32,
    compressor_ratio_hundredths: AtomicU32,
    compressor_threshold_tenth_db: AtomicI32,
    compressor_gain_tenth_db: AtomicI32,
    compressor_release_tenth_ms: AtomicU32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgcPreset {
    Disabled,
    Light,
    Normal,
    Strong,
    Voice,
}

impl AgcPreset {
    pub const ALL: [Self; 5] = [
        Self::Disabled,
        Self::Light,
        Self::Normal,
        Self::Strong,
        Self::Voice,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "Disabled",
            Self::Light => "Light",
            Self::Normal => "Normal",
            Self::Strong => "Strong",
            Self::Voice => "Voice",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|preset| preset.as_str() == value)
    }

    fn from_index(index: u32) -> Self {
        match index {
            1 => Self::Light,
            2 => Self::Normal,
            3 => Self::Strong,
            4 => Self::Voice,
            _ => Self::Disabled,
        }
    }

    fn index(self) -> u32 {
        match self {
            Self::Disabled => 0,
            Self::Light => 1,
            Self::Normal => 2,
            Self::Strong => 3,
            Self::Voice => 4,
        }
    }

    fn params(self) -> Option<AgcParams> {
        match self {
            Self::Disabled => None,
            Self::Light => Some(AgcParams {
                target_db: -18.0,
                gate_db: -46.0,
                max_gain_db: 6.0,
                min_gain_db: -6.0,
                rise_ms: 3500.0,
                fall_ms: 450.0,
            }),
            Self::Normal => Some(AgcParams {
                target_db: -16.0,
                gate_db: -48.0,
                max_gain_db: 10.0,
                min_gain_db: -8.0,
                rise_ms: 2200.0,
                fall_ms: 300.0,
            }),
            Self::Strong => Some(AgcParams {
                target_db: -14.0,
                gate_db: -50.0,
                max_gain_db: 14.0,
                min_gain_db: -10.0,
                rise_ms: 1300.0,
                fall_ms: 180.0,
            }),
            Self::Voice => Some(AgcParams {
                target_db: -17.0,
                gate_db: -42.0,
                max_gain_db: 12.0,
                min_gain_db: -8.0,
                rise_ms: 900.0,
                fall_ms: 140.0,
            }),
        }
    }
}

impl AudioProcessingSettings {
    fn set_bypassed(&self, bypassed: bool) {
        self.bypassed.store(bypassed, Ordering::Relaxed);
    }

    fn bypassed(&self) -> bool {
        self.bypassed.load(Ordering::Relaxed)
    }

    fn set_master_volume_percent(&self, volume: f32) {
        let per_mille = (volume.clamp(0.0, 100.0) * 10.0).round() as u32;
        self.master_volume_per_mille
            .store(per_mille, Ordering::Relaxed);
    }

    fn master_volume_percent(&self) -> f32 {
        self.master_volume_per_mille.load(Ordering::Relaxed) as f32 / 10.0
    }

    fn master_gain(&self) -> f32 {
        self.master_volume_per_mille.load(Ordering::Relaxed) as f32 / 1000.0
    }

    fn set_agc_preset(&self, preset: AgcPreset) {
        self.agc_preset.store(preset.index(), Ordering::Relaxed);
    }

    fn agc_preset(&self) -> AgcPreset {
        AgcPreset::from_index(self.agc_preset.load(Ordering::Relaxed))
    }

    fn agc_params(&self) -> Option<AgcParams> {
        self.agc_preset().params()
    }

    pub fn set_eq_enabled(&self, enabled: bool) {
        self.eq_enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn eq_enabled(&self) -> bool {
        self.eq_enabled.load(Ordering::Relaxed)
    }

    pub fn set_eq_gain_db(&self, band: usize, gain_db: f32) {
        if let Some(gain) = self.eq_gains_tenth_db.get(band) {
            let tenth_db = (gain_db.clamp(-15.0, 15.0) * 10.0).round() as i32;
            gain.store(tenth_db, Ordering::Relaxed);
        }
    }

    pub fn eq_gains_db(&self) -> Vec<f32> {
        self.eq_gains_tenth_db
            .iter()
            .map(|gain| gain.load(Ordering::Relaxed) as f32 / 10.0)
            .collect()
    }

    fn eq_gains_tenth_db(&self) -> [i32; 10] {
        std::array::from_fn(|idx| self.eq_gains_tenth_db[idx].load(Ordering::Relaxed))
    }

    fn set_compressor_attack_ms(&self, value: f32) {
        self.compressor_attack_tenth_ms.store(
            (value.clamp(0.1, 5000.0) * 10.0).round() as u32,
            Ordering::Relaxed,
        );
    }

    fn compressor_attack_ms(&self) -> f32 {
        self.compressor_attack_tenth_ms.load(Ordering::Relaxed) as f32 / 10.0
    }

    fn set_compressor_ratio(&self, value: f32) {
        self.compressor_ratio_hundredths.store(
            (value.clamp(1.0, 40.0) * 100.0).round() as u32,
            Ordering::Relaxed,
        );
    }

    fn compressor_ratio(&self) -> f32 {
        self.compressor_ratio_hundredths.load(Ordering::Relaxed) as f32 / 100.0
    }

    fn set_compressor_threshold_db(&self, value: f32) {
        self.compressor_threshold_tenth_db.store(
            (value.clamp(-80.0, 0.0) * 10.0).round() as i32,
            Ordering::Relaxed,
        );
    }

    fn compressor_threshold_db(&self) -> f32 {
        self.compressor_threshold_tenth_db.load(Ordering::Relaxed) as f32 / 10.0
    }

    fn set_compressor_gain_db(&self, value: f32) {
        self.compressor_gain_tenth_db.store(
            (value.clamp(-24.0, 24.0) * 10.0).round() as i32,
            Ordering::Relaxed,
        );
    }

    fn compressor_gain_db(&self) -> f32 {
        self.compressor_gain_tenth_db.load(Ordering::Relaxed) as f32 / 10.0
    }

    fn set_compressor_release_ms(&self, value: f32) {
        self.compressor_release_tenth_ms.store(
            (value.clamp(1.0, 10000.0) * 10.0).round() as u32,
            Ordering::Relaxed,
        );
    }

    fn compressor_release_ms(&self) -> f32 {
        self.compressor_release_tenth_ms.load(Ordering::Relaxed) as f32 / 10.0
    }

    fn compressor_params(&self) -> CompressorParams {
        CompressorParams {
            attack_ms: self.compressor_attack_ms(),
            ratio: self.compressor_ratio(),
            threshold_db: self.compressor_threshold_db(),
            makeup_gain_db: self.compressor_gain_db(),
            release_ms: self.compressor_release_ms(),
        }
    }
}

impl Default for AudioProcessingSettings {
    fn default() -> Self {
        Self {
            bypassed: AtomicBool::new(false),
            master_volume_per_mille: AtomicU32::new(1000),
            agc_preset: AtomicU32::new(0),
            eq_enabled: AtomicBool::new(false),
            eq_gains_tenth_db: std::array::from_fn(|_| AtomicI32::new(0)),
            compressor_attack_tenth_ms: AtomicU32::new(250),
            compressor_ratio_hundredths: AtomicU32::new(400),
            compressor_threshold_tenth_db: AtomicI32::new(-270),
            compressor_gain_tenth_db: AtomicI32::new(0),
            compressor_release_tenth_ms: AtomicU32::new(15000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
}

impl AudioDeviceInfo {
    pub fn default_option() -> Self {
        Self {
            id: String::new(),
            name: String::from("(Default)"),
        }
    }

    fn name_fallback(name: String) -> Self {
        Self {
            id: format!("{DEVICE_ID_NAME_PREFIX}{name}"),
            name,
        }
    }
}

impl std::fmt::Display for AudioDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceDirection {
    Input,
    Output,
}

pub fn list_output_devices() -> Vec<AudioDeviceInfo> {
    let host = cpal::default_host();
    let mut devices = Vec::new();
    #[cfg(target_os = "macos")]
    devices.extend(macos_coreaudio_device_infos(DeviceDirection::Output));

    devices.extend(
        host.output_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| d.name().ok())
                    .map(AudioDeviceInfo::name_fallback)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    );
    dedup_device_infos(devices)
}

pub fn list_input_devices() -> Vec<AudioDeviceInfo> {
    let host = cpal::default_host();
    let mut devices = Vec::new();
    #[cfg(target_os = "macos")]
    devices.extend(macos_coreaudio_device_infos(DeviceDirection::Input));

    devices.extend(
        host.input_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| d.name().ok())
                    .map(AudioDeviceInfo::name_fallback)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    );
    dedup_device_infos(devices)
}

pub fn normalize_device_id(stored: &str, devices: &[AudioDeviceInfo]) -> String {
    let stored = stored.trim();
    if stored.is_empty() {
        return String::new();
    }

    if devices.iter().any(|device| device.id == stored) {
        return stored.to_string();
    }

    devices
        .iter()
        .find(|device| device.name == stored)
        .map(|device| device.id.clone())
        .unwrap_or_default()
}

pub fn selected_device_option(stored: &str, devices: &[AudioDeviceInfo]) -> AudioDeviceInfo {
    let id = normalize_device_id(stored, devices);
    if id.is_empty() {
        return AudioDeviceInfo::default_option();
    }

    devices
        .iter()
        .find(|device| device.id == id)
        .cloned()
        .unwrap_or_else(AudioDeviceInfo::default_option)
}

pub fn device_name_from_selector(selector: &str, direction: DeviceDirection) -> Option<String> {
    let selector = selector.trim();
    if selector.is_empty() {
        return None;
    }

    if let Some(name) = selector.strip_prefix(DEVICE_ID_NAME_PREFIX) {
        return Some(name.to_string());
    }

    #[cfg(target_os = "macos")]
    if let Some(uid) = selector.strip_prefix(DEVICE_ID_COREAUDIO_PREFIX) {
        return macos_coreaudio_devices(direction)
            .into_iter()
            .find(|device| device.uid.as_deref() == Some(uid))
            .map(|device| device.name);
    }

    Some(selector.to_string())
}

fn dedup_device_infos(devices: Vec<AudioDeviceInfo>) -> Vec<AudioDeviceInfo> {
    let mut seen_ids = HashSet::new();
    let mut seen_names = HashSet::new();
    devices
        .into_iter()
        .filter_map(|device| {
            let id = device.id.trim();
            let name = device.name.trim();
            if id.is_empty() || name.is_empty() {
                return None;
            }
            let name_key = name.to_lowercase();
            if !seen_ids.insert(id.to_string()) || !seen_names.insert(name_key) {
                None
            } else {
                Some(AudioDeviceInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                })
            }
        })
        .collect()
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
struct MacosCoreAudioDevice {
    name: String,
    uid: Option<String>,
    input_channels: u32,
    output_channels: u32,
    nominal_sample_rate: Option<u32>,
}

#[cfg(target_os = "macos")]
fn macos_coreaudio_device_infos(direction: DeviceDirection) -> Vec<AudioDeviceInfo> {
    macos_coreaudio_devices(direction)
        .into_iter()
        .filter_map(|device| {
            let channels = match direction {
                DeviceDirection::Input => device.input_channels,
                DeviceDirection::Output => device.output_channels,
            };
            if channels == 0 {
                return None;
            }
            let id = device
                .uid
                .map(|uid| format!("{DEVICE_ID_COREAUDIO_PREFIX}{uid}"))
                .unwrap_or_else(|| format!("{DEVICE_ID_NAME_PREFIX}{}", device.name));
            Some(AudioDeviceInfo {
                id,
                name: device.name,
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn macos_coreaudio_devices(_direction: DeviceDirection) -> Vec<MacosCoreAudioDevice> {
    use std::ffi::CStr;
    use std::mem;
    use std::ptr::null;
    use std::slice;

    use core_foundation_sys::string::{
        kCFStringEncodingUTF8, CFStringGetCString, CFStringGetCStringPtr, CFStringRef,
    };
    use coreaudio_sys::{
        kAudioDevicePropertyDeviceNameCFString, kAudioDevicePropertyNominalSampleRate,
        kAudioDevicePropertyScopeInput, kAudioDevicePropertyScopeOutput,
        kAudioDevicePropertyStreamConfiguration, kAudioHardwareNoError,
        kAudioHardwarePropertyDevices, kAudioObjectPropertyElementMaster,
        kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, AudioBuffer, AudioBufferList,
        AudioDeviceID, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
        AudioObjectPropertyAddress,
    };

    const K_AUDIO_DEVICE_PROPERTY_DEVICE_UID: u32 = 0x7569_6420; // 'uid '

    fn property_size(object_id: u32, address: &AudioObjectPropertyAddress) -> Option<u32> {
        let data_size = 0u32;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                object_id,
                address as *const _,
                0,
                null(),
                &data_size as *const _ as *mut _,
            )
        };
        (status == kAudioHardwareNoError as i32).then_some(data_size)
    }

    fn cfstring_property(device_id: AudioDeviceID, selector: u32, scope: u32) -> Option<String> {
        let address = AudioObjectPropertyAddress {
            mSelector: selector,
            mScope: scope,
            mElement: kAudioObjectPropertyElementMaster,
        };
        let value: CFStringRef = null();
        let data_size = mem::size_of::<CFStringRef>() as u32;
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &address as *const _,
                0,
                null(),
                &data_size as *const _ as *mut _,
                &value as *const _ as *mut _,
            )
        };
        if status != kAudioHardwareNoError as i32 || value.is_null() {
            return None;
        }

        let c_string = unsafe { CFStringGetCStringPtr(value, kCFStringEncodingUTF8) };
        if !c_string.is_null() {
            return unsafe { CStr::from_ptr(c_string) }
                .to_str()
                .ok()
                .map(str::to_string);
        }

        let mut buf: [i8; 512] = [0; 512];
        let ok = unsafe {
            CFStringGetCString(
                value,
                buf.as_mut_ptr(),
                buf.len() as _,
                kCFStringEncodingUTF8,
            )
        };
        if ok == 0 {
            None
        } else {
            unsafe { CStr::from_ptr(buf.as_ptr()) }
                .to_str()
                .ok()
                .map(str::to_string)
        }
    }

    fn device_name(device_id: AudioDeviceID) -> Option<String> {
        cfstring_property(
            device_id,
            kAudioDevicePropertyDeviceNameCFString,
            kAudioDevicePropertyScopeOutput,
        )
    }

    fn device_uid(device_id: AudioDeviceID) -> Option<String> {
        cfstring_property(
            device_id,
            K_AUDIO_DEVICE_PROPERTY_DEVICE_UID,
            kAudioObjectPropertyScopeGlobal,
        )
    }

    fn channel_count(device_id: AudioDeviceID, scope: u32) -> u32 {
        let address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: scope,
            mElement: kAudioObjectPropertyElementMaster,
        };
        let Some(mut data_size) = property_size(device_id, &address) else {
            return 0;
        };
        if data_size == 0 {
            return 0;
        }

        let mut bytes = vec![0u8; data_size as usize];
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &address as *const _,
                0,
                null(),
                &mut data_size as *mut _,
                bytes.as_mut_ptr() as *mut _,
            )
        };
        if status != kAudioHardwareNoError as i32 {
            return 0;
        }

        let list = bytes.as_ptr() as *const AudioBufferList;
        if list.is_null() {
            return 0;
        }

        unsafe {
            let buffers =
                slice::from_raw_parts((*list).mBuffers.as_ptr(), (*list).mNumberBuffers as usize);
            buffers
                .iter()
                .map(|buffer: &AudioBuffer| buffer.mNumberChannels)
                .sum()
        }
    }

    fn nominal_sample_rate(device_id: AudioDeviceID) -> Option<u32> {
        let address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyNominalSampleRate,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMaster,
        };
        let mut sample_rate = 0.0f64;
        let mut data_size = mem::size_of::<f64>() as u32;
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &address as *const _,
                0,
                null(),
                &mut data_size as *mut _,
                &mut sample_rate as *mut _ as *mut _,
            )
        };
        if status == kAudioHardwareNoError as i32 && sample_rate.is_finite() && sample_rate > 0.0 {
            Some(sample_rate.round() as u32)
        } else {
            None
        }
    }

    let address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDevices,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };
    let Some(mut data_size) = property_size(kAudioObjectSystemObject, &address) else {
        return Vec::new();
    };
    if data_size == 0 {
        return Vec::new();
    }

    let device_count = data_size as usize / mem::size_of::<AudioDeviceID>();
    let mut device_ids = vec![0 as AudioDeviceID; device_count];
    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &address as *const _,
            0,
            null(),
            &mut data_size as *mut _,
            device_ids.as_mut_ptr() as *mut _,
        )
    };
    if status != kAudioHardwareNoError as i32 {
        return Vec::new();
    }

    device_ids
        .into_iter()
        .filter_map(|device_id| {
            let name = device_name(device_id)?;
            Some(MacosCoreAudioDevice {
                name,
                uid: device_uid(device_id),
                input_channels: channel_count(device_id, kAudioDevicePropertyScopeInput),
                output_channels: channel_count(device_id, kAudioDevicePropertyScopeOutput),
                nominal_sample_rate: nominal_sample_rate(device_id),
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn macos_coreaudio_output_config_for_name(
    name: &str,
    sample_rate: u32,
) -> Option<SupportedStreamConfig> {
    macos_coreaudio_devices(DeviceDirection::Output)
        .into_iter()
        .find(|device| device.name == name && device.output_channels > 0)
        .map(|device| {
            let sample_rate = if sample_rate > 0 {
                sample_rate
            } else {
                device.nominal_sample_rate.unwrap_or(44_100)
            };
            SupportedStreamConfig::new(
                device.output_channels.min(u16::MAX as u32) as u16,
                SampleRate(sample_rate),
                SupportedBufferSize::Unknown,
                SampleFormat::F32,
            )
        })
}

fn get_output_device(
    host: &cpal::Host,
    selector: Option<&str>,
) -> Result<cpal::Device, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(name) = selector
        .and_then(|selector| device_name_from_selector(selector, DeviceDirection::Output))
        .filter(|n| !n.is_empty())
    {
        if let Ok(mut devices) = host.output_devices() {
            if let Some(device) = devices.find(|d| d.name().ok().as_deref() == Some(&name)) {
                return Ok(device);
            }
        }
        if let Ok(mut devices) = host.devices() {
            if let Some(device) = devices.find(|d| d.name().ok().as_deref() == Some(&name)) {
                return Ok(device);
            }
        }
    }
    host.default_output_device()
        .ok_or_else(|| "No audio output device available".into())
}

pub struct AudioPlayer {
    id: PlayerId,
    loaded_path: Option<PathBuf>,
    cue_in: Duration,
    device_name: Option<String>,
    processing: Arc<AudioProcessingSettings>,
    duration: Option<Duration>,
    levels: Arc<AudioLevels>,
    stop_tx: Option<Sender<()>>,
    seek_tx: Option<Sender<SeekRequest>>,
    pause_tx: Option<Sender<bool>>,
    fade_tx: Option<Sender<FadeRequest>>,
    done_rx: Option<mpsc::Receiver<()>>,
    preload_rx: Option<mpsc::Receiver<Option<Preloaded>>>,
    position_ms: Option<Arc<AtomicU64>>,
    paused: bool,
}

impl AudioPlayer {
    fn new(id: PlayerId, processing: Arc<AudioProcessingSettings>) -> Self {
        Self {
            id,
            loaded_path: None,
            cue_in: Duration::ZERO,
            device_name: None,
            processing,
            duration: None,
            levels: Arc::new(AudioLevels::default()),
            stop_tx: None,
            seek_tx: None,
            pause_tx: None,
            fade_tx: None,
            done_rx: None,
            preload_rx: None,
            position_ms: None,
            paused: false,
        }
    }

    pub fn set_device(&mut self, name: Option<String>) {
        self.device_name = name;
    }

    pub fn handle(&mut self, command: PlayerCommand) {
        match command {
            PlayerCommand::Load { path, cue_in } => self.load(path, cue_in),
            PlayerCommand::Play => self.play(),
            PlayerCommand::Pause => self.pause(),
            PlayerCommand::Resume => self.resume(),
            PlayerCommand::TogglePause => self.toggle_pause(),
            PlayerCommand::Stop => self.stop(),
            PlayerCommand::SoftStop(duration) => self.soft_stop(duration),
            PlayerCommand::FadeOut(duration) => self.fade_out(duration),
            PlayerCommand::Restart => self.restart(),
            PlayerCommand::SeekRelative(offset_ms) => self.seek_relative(offset_ms),
        }
    }

    pub fn load(&mut self, path: PathBuf, cue_in: Duration) {
        self.stop_thread();
        self.cue_in = cue_in;
        self.duration = read_duration(&path);
        self.preload_rx = Some(preload(path.clone(), cue_in, self.device_name.clone()));
        self.loaded_path = Some(path);
    }

    pub fn play(&mut self) {
        let Some(path) = self.loaded_path.clone() else {
            return;
        };

        self.stop_thread();
        let preloaded = self
            .preload_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok().flatten());
        self.preload_rx = None;

        let (stop_tx, seek_tx, pause_tx, fade_tx, position_ms, done_rx) = play(
            path,
            preloaded,
            self.cue_in,
            Arc::clone(&self.levels),
            Arc::clone(&self.processing),
            self.id != PlayerId::Preview,
            self.device_name.clone(),
        );
        self.stop_tx = Some(stop_tx);
        self.seek_tx = Some(seek_tx);
        self.pause_tx = Some(pause_tx);
        self.fade_tx = Some(fade_tx);
        self.position_ms = Some(position_ms);
        self.done_rx = Some(done_rx);
        self.paused = false;
    }

    pub fn pause(&mut self) {
        if let Some(tx) = &self.pause_tx {
            let _ = tx.send(true);
            self.paused = true;
            self.levels.reset();
        }
    }

    pub fn resume(&mut self) {
        if let Some(tx) = &self.pause_tx {
            let _ = tx.send(false);
            self.paused = false;
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.stop_tx.is_some() && self.paused {
            self.resume();
        } else if self.stop_tx.is_some() {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn stop(&mut self) {
        self.stop_thread();
        self.preload_rx = self
            .loaded_path
            .as_ref()
            .map(|path| preload(path.clone(), self.cue_in, self.device_name.clone()));
    }

    pub fn soft_stop(&mut self, duration: Duration) {
        self.fade_out(duration);
    }

    pub fn fade_out(&mut self, duration: Duration) {
        if duration.is_zero() || self.paused {
            self.stop();
            return;
        }

        if let Some(tx) = &self.fade_tx {
            let _ = tx.send(FadeRequest { duration });
        } else {
            self.stop();
        }
    }

    pub fn seek_relative(&mut self, offset_ms: i64) {
        if let Some(tx) = &self.seek_tx {
            let _ = tx.send(SeekRequest::Relative(offset_ms));
        }
    }

    pub fn restart(&mut self) {
        if let Some(tx) = &self.seek_tx {
            let _ = tx.send(SeekRequest::Absolute(Duration::ZERO));
        }
    }

    pub fn poll(&mut self) {
        use std::sync::mpsc::TryRecvError;

        let finished = self
            .done_rx
            .as_ref()
            .is_some_and(|rx| !matches!(rx.try_recv(), Err(TryRecvError::Empty)));

        if finished {
            self.stop_tx = None;
            self.seek_tx = None;
            self.pause_tx = None;
            self.fade_tx = None;
            self.done_rx = None;
            self.position_ms = None;
            self.paused = false;
            self.levels.reset();
            self.preload_rx = self
                .loaded_path
                .as_ref()
                .map(|path| preload(path.clone(), self.cue_in, self.device_name.clone()));
        }
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        PlayerSnapshot {
            id: self.id,
            state: self.state(),
            loaded_path: self.loaded_path.clone(),
            position: self.position(),
            duration: self.duration,
        }
    }

    pub fn is_active(&self) -> bool {
        self.stop_tx.is_some()
    }

    pub fn is_playing(&self) -> bool {
        self.stop_tx.is_some() && !self.paused
    }

    pub fn levels(&self) -> (f32, f32) {
        self.levels.read()
    }

    fn state(&self) -> PlayerState {
        if self.stop_tx.is_some() && self.paused {
            PlayerState::Paused
        } else if self.stop_tx.is_some() {
            PlayerState::Playing
        } else if self.loaded_path.is_some() {
            PlayerState::Loaded
        } else {
            PlayerState::Empty
        }
    }

    fn position(&self) -> Duration {
        self.position_ms
            .as_ref()
            .map(|pos| Duration::from_millis(pos.load(Ordering::Relaxed)))
            .unwrap_or_default()
    }

    fn stop_thread(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        self.seek_tx = None;
        self.pause_tx = None;
        self.fade_tx = None;
        self.done_rx = None;
        self.position_ms = None;
        self.paused = false;
        self.levels.reset();
    }
}

#[derive(Default)]
pub struct AudioLevels {
    left: AtomicU32,
    right: AtomicU32,
}

impl AudioLevels {
    fn store(&self, left: f32, right: f32) {
        self.left
            .store(left.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        self.right
            .store(right.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    fn reset(&self) {
        self.store(0.0, 0.0);
    }

    fn read(&self) -> (f32, f32) {
        (
            f32::from_bits(self.left.load(Ordering::Relaxed)),
            f32::from_bits(self.right.load(Ordering::Relaxed)),
        )
    }
}

/// Résultat du pré-chargement : décodeur prêt + buffer pré-rempli avec ~0.5 s d'audio.
pub struct Preloaded {
    samples: Vec<f32>,
    format: Box<dyn symphonia::core::formats::FormatReader>,
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    track_id: u32,
    source_sample_rate: u32,
}

struct AudioChunk {
    generation: u64,
    samples: Vec<f32>,
}

enum SeekRequest {
    Relative(i64),
    Absolute(Duration),
}

struct FadeRequest {
    duration: Duration,
}

#[derive(Default)]
struct FadeState {
    active: bool,
    completed: bool,
    total_frames: u64,
    elapsed_frames: u64,
    start_gain: f32,
}

impl FadeState {
    fn start(&mut self, duration: Duration, sample_rate: u32) {
        let total_frames = (duration.as_secs_f64() * sample_rate as f64).ceil() as u64;
        if total_frames == 0 {
            self.active = false;
            self.completed = true;
            self.total_frames = 0;
            self.elapsed_frames = 0;
            self.start_gain = 0.0;
            return;
        }

        self.start_gain = self.current_gain();
        self.active = true;
        self.completed = false;
        self.total_frames = total_frames;
        self.elapsed_frames = 0;
    }

    fn current_gain(&self) -> f32 {
        if self.completed {
            0.0
        } else if self.active && self.total_frames > 0 {
            let progress = self.elapsed_frames as f32 / self.total_frames as f32;
            self.start_gain * (1.0 - progress).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

struct EqProcessor {
    sample_rate: f32,
    filters: Vec<[Biquad; 10]>,
    current_gains_tenth_db: [i32; 10],
    current_enabled: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct AgcParams {
    target_db: f32,
    gate_db: f32,
    max_gain_db: f32,
    min_gain_db: f32,
    rise_ms: f32,
    fall_ms: f32,
}

struct AgcProcessor {
    sample_rate: f32,
    current_gain: f32,
    current_params: Option<AgcParams>,
    rise_coeff: f32,
    fall_coeff: f32,
}

impl AgcProcessor {
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            current_gain: 1.0,
            current_params: None,
            rise_coeff: 0.0,
            fall_coeff: 0.0,
        }
    }

    fn process(&mut self, data: &mut [f32], channels: usize, processing: &AudioProcessingSettings) {
        let params = processing.agc_params();
        if params != self.current_params {
            self.update_params(params);
        }

        let Some(params) = self.current_params else {
            return;
        };

        let channels = channels.max(1);
        for frame in data.chunks_mut(channels) {
            let mean_square =
                frame.iter().map(|sample| sample * sample).sum::<f32>() / frame.len().max(1) as f32;
            let rms = mean_square.sqrt().max(0.000_001);
            let level_db = 20.0 * rms.log10();

            let target_gain = if level_db < params.gate_db {
                1.0
            } else {
                db_to_gain(
                    (params.target_db - level_db).clamp(params.min_gain_db, params.max_gain_db),
                )
            };

            let coeff = if target_gain > self.current_gain {
                self.rise_coeff
            } else {
                self.fall_coeff
            };
            self.current_gain = target_gain + coeff * (self.current_gain - target_gain);

            for sample in frame {
                *sample = (*sample * self.current_gain).clamp(-4.0, 4.0);
            }
        }
    }

    fn update_params(&mut self, params: Option<AgcParams>) {
        self.current_params = params;
        if let Some(params) = params {
            self.rise_coeff = smoothing_coeff(params.rise_ms, self.sample_rate);
            self.fall_coeff = smoothing_coeff(params.fall_ms, self.sample_rate);
        } else {
            self.reset();
        }
    }

    fn reset(&mut self) {
        self.current_gain = 1.0;
    }
}

impl EqProcessor {
    fn new(sample_rate: u32, channels: usize) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            filters: vec![[Biquad::default(); 10]; channels.max(1)],
            current_gains_tenth_db: [i32::MIN; 10],
            current_enabled: false,
        }
    }

    fn process(&mut self, data: &mut [f32], processing: &AudioProcessingSettings) {
        let enabled = processing.eq_enabled();
        if !enabled {
            if self.current_enabled {
                self.reset();
                self.current_enabled = false;
            }
            return;
        }

        let gains = processing.eq_gains_tenth_db();
        if !self.current_enabled || gains != self.current_gains_tenth_db {
            self.update_coefficients(gains);
            self.current_enabled = true;
        }

        let channels = self.filters.len();
        for frame in data.chunks_mut(channels) {
            for (channel, sample) in frame.iter_mut().enumerate() {
                let mut value = *sample;
                for filter in &mut self.filters[channel] {
                    value = filter.process(value);
                }
                *sample = value.clamp(-4.0, 4.0);
            }
        }
    }

    fn update_coefficients(&mut self, gains: [i32; 10]) {
        self.current_gains_tenth_db = gains;
        for (band, gain_tenth_db) in gains.into_iter().enumerate() {
            let coeffs = BiquadCoefficients::peaking_eq(
                EQ_BAND_FREQS[band],
                self.sample_rate,
                1.2,
                gain_tenth_db as f32 / 10.0,
            );

            for channel_filters in &mut self.filters {
                channel_filters[band].set_coefficients(coeffs);
            }
        }
    }

    fn reset(&mut self) {
        for channel_filters in &mut self.filters {
            for filter in channel_filters {
                filter.reset();
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Biquad {
    coefficients: BiquadCoefficients,
    z1: f32,
    z2: f32,
}

impl Biquad {
    fn set_coefficients(&mut self, coefficients: BiquadCoefficients) {
        self.coefficients = coefficients;
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.coefficients.b0 * input + self.z1;
        self.z1 = self.coefficients.b1 * input - self.coefficients.a1 * output + self.z2;
        self.z2 = self.coefficients.b2 * input - self.coefficients.a2 * output;
        output
    }

    fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

impl Default for Biquad {
    fn default() -> Self {
        Self {
            coefficients: BiquadCoefficients::identity(),
            z1: 0.0,
            z2: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadCoefficients {
    fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    fn peaking_eq(freq_hz: f32, sample_rate: f32, q: f32, gain_db: f32) -> Self {
        if gain_db.abs() < 0.05 || freq_hz >= sample_rate * 0.45 {
            return Self::identity();
        }

        let omega = 2.0 * std::f32::consts::PI * freq_hz / sample_rate;
        let alpha = omega.sin() / (2.0 * q.max(0.1));
        let amp = 10.0_f32.powf(gain_db / 40.0);
        let cos_omega = omega.cos();

        let b0 = 1.0 + alpha * amp;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * amp;
        let a0 = 1.0 + alpha / amp;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / amp;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct CompressorParams {
    attack_ms: f32,
    ratio: f32,
    threshold_db: f32,
    makeup_gain_db: f32,
    release_ms: f32,
}

struct Compressor {
    sample_rate: f32,
    current_gain: f32,
    params: CompressorParams,
    attack_coeff: f32,
    release_coeff: f32,
}

impl Compressor {
    fn new(sample_rate: u32) -> Self {
        let params = CompressorParams {
            attack_ms: 25.0,
            ratio: 4.0,
            threshold_db: -27.0,
            makeup_gain_db: 0.0,
            release_ms: 1500.0,
        };
        let mut compressor = Self {
            sample_rate: sample_rate as f32,
            current_gain: 1.0,
            params,
            attack_coeff: 0.0,
            release_coeff: 0.0,
        };
        compressor.update_params(params);
        compressor
    }

    fn process(&mut self, data: &mut [f32], channels: usize, processing: &AudioProcessingSettings) {
        let params = processing.compressor_params();
        if params != self.params {
            self.update_params(params);
        }

        if self.params.ratio <= 1.01 && self.params.makeup_gain_db.abs() < 0.05 {
            return;
        }

        let channels = channels.max(1);
        for frame in data.chunks_mut(channels) {
            let peak = frame
                .iter()
                .map(|sample| sample.abs())
                .fold(0.0_f32, f32::max)
                .max(0.000_001);
            let level_db = 20.0 * peak.log10();
            let gain_reduction_db = if level_db > self.params.threshold_db {
                let compressed_db = self.params.threshold_db
                    + (level_db - self.params.threshold_db) / self.params.ratio;
                compressed_db - level_db
            } else {
                0.0
            };
            let target_gain = db_to_gain(gain_reduction_db + self.params.makeup_gain_db);
            let coeff = if target_gain < self.current_gain {
                self.attack_coeff
            } else {
                self.release_coeff
            };
            self.current_gain = target_gain + coeff * (self.current_gain - target_gain);

            for sample in frame {
                *sample = (*sample * self.current_gain).clamp(-4.0, 4.0);
            }
        }
    }

    fn update_params(&mut self, params: CompressorParams) {
        self.params = params;
        self.attack_coeff = smoothing_coeff(params.attack_ms, self.sample_rate);
        self.release_coeff = smoothing_coeff(params.release_ms, self.sample_rate);
    }

    fn reset(&mut self) {
        self.current_gain = 1.0;
    }
}

fn smoothing_coeff(time_ms: f32, sample_rate: f32) -> f32 {
    (-1.0 / ((time_ms.max(0.1) / 1000.0) * sample_rate.max(1.0))).exp()
}

fn db_to_gain(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Lit uniquement les métadonnées pour obtenir la durée totale du fichier.
pub fn read_duration(path: &std::path::Path) -> Option<std::time::Duration> {
    let file = std::fs::File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;
    let track = probed.format.default_track()?;
    let n_frames = track.codec_params.n_frames?;
    let sample_rate = track.codec_params.sample_rate? as f64;
    Some(std::time::Duration::from_secs_f64(
        n_frames as f64 / sample_rate,
    ))
}

/// Démarre le pré-chargement en arrière-plan dès la sélection du fichier.
/// Retourne un Receiver : `try_recv()` donne le résultat quand il est prêt.
pub fn preload(
    path: PathBuf,
    cue_in: Duration,
    device_name: Option<String>,
) -> mpsc::Receiver<Option<Preloaded>> {
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _ = tx.send(do_preload(path, cue_in, device_name).ok());
    });
    rx
}

fn do_preload(
    path: PathBuf,
    cue_in: Duration,
    device_name: Option<String>,
) -> Result<Preloaded, Box<dyn std::error::Error + Send + Sync>> {
    let file = std::fs::File::open(&path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = symphonia::default::get_probe().format(
        &Hint::new(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;
    let track = format.default_track().ok_or("No audio track")?;
    let track_id = track.id;
    let source_sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    // Déterminer la config cpal pour mixer correctement dès le pré-chargement
    let host = cpal::default_host();
    let device = get_output_device(&host, device_name.as_deref())?;
    let out_channels = config_for(&device, source_sample_rate)?.channels() as usize;

    if !cue_in.is_zero() {
        let target_secs = cue_in.as_secs_f64();
        let target_time = Time {
            seconds: target_secs as u64,
            frac: target_secs.fract(),
        };
        let _ = format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: target_time,
                track_id: Some(track_id),
            },
        );
        decoder.reset();
    }

    let mut samples = Vec::new();
    let prefill_target = source_sample_rate as usize * out_channels / 2; // 0.5 s

    loop {
        if samples.len() >= prefill_target {
            break;
        }
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => samples.extend(decoded_to_samples(decoded, out_channels)),
            Err(_) => continue,
        }
    }

    Ok(Preloaded {
        samples,
        format,
        decoder,
        track_id,
        source_sample_rate,
    })
}

/// Lance la lecture. Si `preloaded` est fourni, le buffer est déjà prêt → Play instantané.
/// Retourne `(stop_tx, seek_tx, pause_tx, position_ms, done_rx)`.
/// - `stop_tx`  : envoyer `()` arrête et remet à zéro
/// - `pause_tx` : envoyer `true` pour pause, `false` pour reprendre
fn play(
    path: PathBuf,
    preloaded: Option<Preloaded>,
    cue_in: Duration,
    levels: Arc<AudioLevels>,
    processing: Arc<AudioProcessingSettings>,
    processing_enabled: bool,
    device_name: Option<String>,
) -> (
    Sender<()>,
    Sender<SeekRequest>,
    Sender<bool>,
    Sender<FadeRequest>,
    Arc<AtomicU64>,
    mpsc::Receiver<()>,
) {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (seek_tx, seek_rx) = mpsc::channel::<SeekRequest>();
    let (pause_tx, pause_rx) = mpsc::channel::<bool>();
    let (fade_tx, fade_rx) = mpsc::channel::<FadeRequest>();
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let position_ms = Arc::new(AtomicU64::new(0));
    let position_ms_thread = Arc::clone(&position_ms);
    std::thread::spawn(move || {
        if let Err(e) = run(
            path,
            preloaded,
            cue_in,
            stop_rx,
            seek_rx,
            pause_rx,
            fade_rx,
            position_ms_thread,
            levels,
            processing,
            processing_enabled,
            device_name,
        ) {
            eprintln!("Audio error: {e}");
        }
        let _ = done_tx.send(());
    });
    (stop_tx, seek_tx, pause_tx, fade_tx, position_ms, done_rx)
}

fn run(
    path: PathBuf,
    preloaded: Option<Preloaded>,
    cue_in: Duration,
    stop_rx: mpsc::Receiver<()>,
    seek_rx: mpsc::Receiver<SeekRequest>,
    pause_rx: mpsc::Receiver<bool>,
    fade_rx: mpsc::Receiver<FadeRequest>,
    position_ms: Arc<AtomicU64>,
    levels: Arc<AudioLevels>,
    processing: Arc<AudioProcessingSettings>,
    processing_enabled: bool,
    device_name: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Récupérer l'état symphonia — depuis le pré-chargement ou en ouvrant le fichier
    let (mut format, mut decoder, track_id, source_sample_rate, preloaded_samples) = match preloaded
    {
        Some(p) => (
            p.format,
            p.decoder,
            p.track_id,
            p.source_sample_rate,
            p.samples,
        ),
        None => {
            let file = std::fs::File::open(&path)?;
            let mss = MediaSourceStream::new(Box::new(file), Default::default());
            let probed = symphonia::default::get_probe().format(
                &Hint::new(),
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )?;
            let mut fmt = probed.format;
            let track = fmt.default_track().ok_or("No audio track")?;
            let track_id = track.id;
            let sr = track.codec_params.sample_rate.unwrap_or(44100);
            let mut dec = symphonia::default::get_codecs()
                .make(&track.codec_params, &DecoderOptions::default())?;
            if !cue_in.is_zero() {
                let target_secs = cue_in.as_secs_f64();
                let target_time = Time {
                    seconds: target_secs as u64,
                    frac: target_secs.fract(),
                };
                if fmt
                    .seek(
                        SeekMode::Coarse,
                        SeekTo::Time {
                            time: target_time,
                            track_id: Some(track_id),
                        },
                    )
                    .is_ok()
                {
                    dec.reset();
                    position_ms.store((target_secs * 1000.0) as u64, Ordering::Relaxed);
                }
            }
            (fmt, dec, track_id, sr, Vec::new())
        }
    };

    let host = cpal::default_host();
    let device = get_output_device(&host, device_name.as_deref())?;
    let config = config_for(&device, source_sample_rate)?;
    let out_channels = config.channels() as usize;
    let output_sample_rate = config.sample_rate().0;
    let (chunk_tx, chunk_rx) = mpsc::sync_channel::<AudioChunk>(64);
    let buffered_samples = Arc::new(AtomicUsize::new(preloaded_samples.len()));
    let playback_generation = Arc::new(AtomicU64::new(0));
    let fade_finished = Arc::new(AtomicBool::new(false));

    let levels_cb = Arc::clone(&levels);
    let processing_cb = Arc::clone(&processing);
    let buffered_samples_cb = Arc::clone(&buffered_samples);
    let playback_generation_cb = Arc::clone(&playback_generation);
    let fade_finished_cb = Arc::clone(&fade_finished);
    let mut current_chunk = preloaded_samples;
    let mut current_offset = 0;
    let mut callback_generation = 0;
    let mut fade_state = FadeState::default();
    let mut agc_processor = AgcProcessor::new(output_sample_rate);
    let mut eq_processor = EqProcessor::new(output_sample_rate, out_channels);
    let mut compressor = Compressor::new(output_sample_rate);
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            fill_output_from_chunks(
                data,
                &chunk_rx,
                &buffered_samples_cb,
                &playback_generation_cb,
                &mut current_chunk,
                &mut current_offset,
                &mut callback_generation,
            );
            apply_fade_out(
                data,
                out_channels,
                output_sample_rate,
                &fade_rx,
                &mut fade_state,
                &fade_finished_cb,
            );
            if !processing_enabled || processing_cb.bypassed() {
                agc_processor.reset();
                eq_processor.reset();
                compressor.reset();
            } else {
                agc_processor.process(data, out_channels, &processing_cb);
                eq_processor.process(data, &processing_cb);
                compressor.process(data, out_channels, &processing_cb);
                apply_master_volume(data, &processing_cb);
            }
            update_levels(&levels_cb, data, out_channels);
        },
        |err| eprintln!("CPAL error: {err}"),
        None,
    )?;
    // Le buffer contient déjà ~0.5 s si pré-chargé : premier son immédiat
    stream.play()?;

    let mut current_ts: u64 = 0;
    let mut paused = false;

    loop {
        if stop_rx.try_recv().is_ok() || fade_finished.load(Ordering::Relaxed) {
            return Ok(());
        }

        if let Ok(should_pause) = pause_rx.try_recv() {
            if should_pause && !paused {
                stream.pause().ok();
                levels.reset();
                paused = true;
            } else if !should_pause && paused {
                stream.play().ok();
                paused = false;
            }
        }

        if paused {
            std::thread::sleep(std::time::Duration::from_millis(20));
            continue;
        }

        if let Ok(seek) = seek_rx.try_recv() {
            let audible_secs = position_ms.load(Ordering::Relaxed) as f64 / 1000.0;
            let target_secs = match seek {
                SeekRequest::Relative(offset_ms) => {
                    (audible_secs + offset_ms as f64 / 1000.0).max(0.0)
                }
                SeekRequest::Absolute(position) => position.as_secs_f64(),
            };
            let target_time = Time {
                seconds: target_secs as u64,
                frac: target_secs.fract(),
            };
            if format
                .seek(
                    SeekMode::Coarse,
                    SeekTo::Time {
                        time: target_time,
                        track_id: Some(track_id),
                    },
                )
                .is_ok()
            {
                decoder.reset();
                current_ts = (target_secs * source_sample_rate as f64) as u64;
                position_ms.store((target_secs * 1000.0).max(0.0) as u64, Ordering::Relaxed);
                buffered_samples.store(0, Ordering::Relaxed);
                playback_generation.fetch_add(1, Ordering::Relaxed);
            }
            continue;
        }

        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }
        current_ts = packet.ts();

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::DecodeError(msg)) => {
                eprintln!("Decode warning: {msg}");
                continue;
            }
            Err(e) => return Err(Box::new(e)),
        };

        let samples = decoded_to_samples(decoded, out_channels);
        let sample_count = samples.len();
        let generation = playback_generation.load(Ordering::Relaxed);
        let chunk = AudioChunk {
            generation,
            samples,
        };

        if !send_audio_chunk(
            &chunk_tx,
            chunk,
            &stop_rx,
            &fade_finished,
            &buffered_samples,
            sample_count,
        ) {
            return Ok(());
        }

        // Position affichée = position décodée − contenu du buffer (audio pas encore joué)
        update_position_from_buffer(
            &position_ms,
            current_ts,
            source_sample_rate,
            out_channels,
            buffered_samples.load(Ordering::Relaxed),
        );
    }

    // Arrêt explicite : vider immédiatement
    if stop_rx.try_recv().is_ok() {
        return Ok(());
    }

    // Fin naturelle : attendre que le callback cpal ait consommé tous les blocs envoyés.
    loop {
        if stop_rx.try_recv().is_ok() || fade_finished.load(Ordering::Relaxed) {
            return Ok(());
        }
        let remaining = buffered_samples.load(Ordering::Relaxed);
        if remaining == 0 {
            break;
        }
        update_position_from_buffer(
            &position_ms,
            current_ts,
            source_sample_rate,
            out_channels,
            remaining,
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Laisser le ring buffer hardware (CoreAudio / WASAPI) terminer sa lecture
    // avant de dropper le stream — sinon les dernières ~100 ms sont coupées
    for _ in 0..20 {
        if stop_rx.try_recv().is_ok() || fade_finished.load(Ordering::Relaxed) {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}

fn fill_output_from_chunks(
    data: &mut [f32],
    chunk_rx: &mpsc::Receiver<AudioChunk>,
    buffered_samples: &AtomicUsize,
    playback_generation: &AtomicU64,
    current_chunk: &mut Vec<f32>,
    current_offset: &mut usize,
    callback_generation: &mut u64,
) {
    let active_generation = playback_generation.load(Ordering::Relaxed);
    if *callback_generation != active_generation {
        current_chunk.clear();
        *current_offset = 0;
        *callback_generation = active_generation;
    }

    let mut written = 0;
    while written < data.len() {
        if *current_offset >= current_chunk.len() {
            current_chunk.clear();
            *current_offset = 0;

            match chunk_rx.try_recv() {
                Ok(chunk) if chunk.generation == active_generation => {
                    *current_chunk = chunk.samples;
                }
                Ok(_) => continue,
                Err(_) => {
                    data[written..].fill(0.0);
                    break;
                }
            }
        }

        let available = current_chunk.len().saturating_sub(*current_offset);
        let to_copy = available.min(data.len() - written);
        if to_copy == 0 {
            continue;
        }

        let src_end = *current_offset + to_copy;
        let dst_end = written + to_copy;
        data[written..dst_end].copy_from_slice(&current_chunk[*current_offset..src_end]);
        decrement_buffered_samples(buffered_samples, to_copy);
        *current_offset = src_end;
        written = dst_end;
    }
}

fn apply_fade_out(
    data: &mut [f32],
    channels: usize,
    sample_rate: u32,
    fade_rx: &mpsc::Receiver<FadeRequest>,
    fade_state: &mut FadeState,
    fade_finished: &AtomicBool,
) {
    while let Ok(request) = fade_rx.try_recv() {
        fade_state.start(request.duration, sample_rate);
    }

    if fade_state.completed {
        data.fill(0.0);
        fade_finished.store(true, Ordering::Relaxed);
        return;
    }

    if !fade_state.active {
        return;
    }

    let channels = channels.max(1);
    let frame_count = data.len() / channels;

    for frame_index in 0..frame_count {
        if fade_state.completed {
            data[frame_index * channels..].fill(0.0);
            fade_finished.store(true, Ordering::Relaxed);
            return;
        }

        let gain = fade_state.current_gain();
        let frame_start = frame_index * channels;
        let frame_end = frame_start + channels;
        for sample in &mut data[frame_start..frame_end] {
            *sample *= gain;
        }

        fade_state.elapsed_frames += 1;
        if fade_state.elapsed_frames >= fade_state.total_frames {
            fade_state.active = false;
            fade_state.completed = true;
        }
    }

    if fade_state.completed {
        fade_finished.store(true, Ordering::Relaxed);
    }
}

fn apply_master_volume(data: &mut [f32], processing: &AudioProcessingSettings) {
    let gain = processing.master_gain();
    if (gain - 1.0).abs() <= f32::EPSILON {
        return;
    }

    for sample in data {
        *sample *= gain;
    }
}

fn decrement_buffered_samples(buffered_samples: &AtomicUsize, consumed: usize) {
    let _ = buffered_samples.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_sub(consumed))
    });
}

fn send_audio_chunk(
    chunk_tx: &SyncSender<AudioChunk>,
    mut chunk: AudioChunk,
    stop_rx: &mpsc::Receiver<()>,
    fade_finished: &AtomicBool,
    buffered_samples: &AtomicUsize,
    sample_count: usize,
) -> bool {
    loop {
        if fade_finished.load(Ordering::Relaxed) {
            return false;
        }

        buffered_samples.fetch_add(sample_count, Ordering::Relaxed);
        match chunk_tx.try_send(chunk) {
            Ok(()) => return true,
            Err(TrySendError::Full(returned_chunk)) => {
                decrement_buffered_samples(buffered_samples, sample_count);
                if stop_rx.try_recv().is_ok() || fade_finished.load(Ordering::Relaxed) {
                    return false;
                }
                chunk = returned_chunk;
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(TrySendError::Disconnected(_)) => {
                decrement_buffered_samples(buffered_samples, sample_count);
                return false;
            }
        }
    }
}

fn update_position_from_buffer(
    position_ms: &AtomicU64,
    current_ts: u64,
    source_sample_rate: u32,
    out_channels: usize,
    buffered_sample_count: usize,
) {
    let buf_ms = buffered_sample_count as u64 * 1000
        / (source_sample_rate as u64 * out_channels.max(1) as u64);
    let decode_ms = current_ts * 1000 / source_sample_rate as u64;
    position_ms.store(decode_ms.saturating_sub(buf_ms), Ordering::Relaxed);
}

fn update_levels(levels: &AudioLevels, data: &[f32], channels: usize) {
    if channels == 0 {
        levels.reset();
        return;
    }

    let mut left = 0.0_f32;
    let mut right = 0.0_f32;

    for frame in data.chunks(channels) {
        let l = frame.first().copied().unwrap_or(0.0).abs();
        let r = frame.get(1).copied().unwrap_or(l).abs();
        left = left.max(l);
        right = right.max(r);
    }

    levels.store(left, right);
}

fn decoded_to_samples(decoded: AudioBufferRef<'_>, out_channels: usize) -> Vec<f32> {
    let spec = *decoded.spec();
    let capacity = decoded.capacity() as u64;
    let mut sample_buf = SampleBuffer::<f32>::new(capacity, spec);
    sample_buf.copy_interleaved_ref(decoded);
    let samples = sample_buf.samples();
    let src_ch = spec.channels.count();

    if src_ch == out_channels {
        samples.to_vec()
    } else if src_ch == 1 {
        let mut out = Vec::with_capacity(samples.len() * out_channels);
        for &s in samples {
            for _ in 0..out_channels {
                out.push(s);
            }
        }
        out
    } else {
        let frame_count = samples.len() / src_ch;
        let mut out = Vec::with_capacity(frame_count * out_channels);
        for frame in samples.chunks(src_ch) {
            for i in 0..out_channels {
                out.push(*frame.get(i % src_ch).unwrap_or(&0.0));
            }
        }
        out
    }
}

fn config_for(
    device: &cpal::Device,
    sample_rate: u32,
) -> Result<SupportedStreamConfig, Box<dyn std::error::Error + Send + Sync>> {
    match device.supported_output_configs() {
        Ok(configs) => {
            for cfg in configs {
                if cfg.min_sample_rate().0 <= sample_rate && sample_rate <= cfg.max_sample_rate().0
                {
                    return Ok(cfg.with_sample_rate(SampleRate(sample_rate)));
                }
            }
        }
        Err(err) => {
            eprintln!("Warning: supported output configs unavailable: {err}");
        }
    }

    eprintln!(
        "Warning: {} Hz is not supported, falling back to the default config",
        sample_rate
    );

    if let Ok(config) = device.default_output_config() {
        return Ok(config);
    }

    #[cfg(target_os = "macos")]
    if let Ok(name) = device.name() {
        if let Some(config) = macos_coreaudio_output_config_for_name(&name, sample_rate) {
            return Ok(config);
        }
    }

    Err("No usable audio output config available".into())
}
