use std::ffi::{c_char, c_int, c_uchar, c_void, CString};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::encoder::EncodedAudioFormat;
use super::StreamingConfig;

type Shout = c_void;
type ShoutMetadata = c_void;

const SHOUT_FORMAT_MP3: c_int = 1;
const SHOUT_PROTOCOL_HTTP: c_int = 0;

extern "C" {
    fn shout_init();
    fn shout_new() -> *mut Shout;
    fn shout_free(self_: *mut Shout);
    fn shout_set_host(self_: *mut Shout, host: *const c_char) -> c_int;
    fn shout_set_protocol(self_: *mut Shout, protocol: c_int) -> c_int;
    fn shout_set_port(self_: *mut Shout, port: c_int) -> c_int;
    fn shout_set_password(self_: *mut Shout, password: *const c_char) -> c_int;
    fn shout_set_mount(self_: *mut Shout, mount: *const c_char) -> c_int;
    fn shout_set_user(self_: *mut Shout, user: *const c_char) -> c_int;
    fn shout_set_format(self_: *mut Shout, format: c_int) -> c_int;
    fn shout_open(self_: *mut Shout) -> c_int;
    fn shout_close(self_: *mut Shout) -> c_int;
    fn shout_send(self_: *mut Shout, data: *const c_uchar, len: usize) -> c_int;
    fn shout_sync(self_: *mut Shout);
    fn shout_metadata_new() -> *mut ShoutMetadata;
    fn shout_metadata_free(self_: *mut ShoutMetadata);
    fn shout_metadata_add(
        self_: *mut ShoutMetadata,
        name: *const c_char,
        value: *const c_char,
    ) -> c_int;
    fn shout_set_metadata(self_: *mut Shout, metadata: *mut ShoutMetadata) -> c_int;
    fn shout_get_error(self_: *mut Shout) -> *const c_char;
}

pub struct IcecastClient {
    inner: IcecastClientInner,
}

enum IcecastClientInner {
    Libshout(LibshoutClient),
    Http(HttpIcecastClient),
}

impl IcecastClient {
    pub fn connect(config: &StreamingConfig, format: EncodedAudioFormat) -> Result<Self, String> {
        let inner = match format {
            EncodedAudioFormat::Mp3 => {
                IcecastClientInner::Libshout(LibshoutClient::connect(config)?)
            }
            EncodedAudioFormat::Aac => {
                IcecastClientInner::Http(HttpIcecastClient::connect(config)?)
            }
        };
        Ok(Self { inner })
    }

    pub fn send(&mut self, data: &[u8]) -> Result<(), String> {
        match &mut self.inner {
            IcecastClientInner::Libshout(client) => client.send(data),
            IcecastClientInner::Http(client) => client.send(data),
        }
    }

    pub fn set_song_title(&mut self, title: &str) -> Result<(), String> {
        match &mut self.inner {
            IcecastClientInner::Libshout(client) => client.set_song_title(title),
            IcecastClientInner::Http(client) => client.set_song_title(title),
        }
    }
}

struct LibshoutClient {
    shout: *mut Shout,
}

impl LibshoutClient {
    fn connect(config: &StreamingConfig) -> Result<Self, String> {
        unsafe {
            shout_init();
        }

        let shout = unsafe { shout_new() };
        if shout.is_null() {
            return Err(String::from("shout_new returned null"));
        }

        let client = Self { shout };
        client.set_string(shout_set_host, &config.host)?;
        client.set_string(shout_set_user, "source")?;
        client.set_string(shout_set_password, &config.password)?;
        client.set_string(shout_set_mount, &config.mountpoint)?;
        client.set_int(shout_set_protocol, SHOUT_PROTOCOL_HTTP)?;
        client.set_int(shout_set_format, SHOUT_FORMAT_MP3)?;
        client.set_int(shout_set_port, config.port)?;

        let opened = unsafe { shout_open(shout) };
        if opened != 0 {
            return Err(client.error());
        }

        Ok(client)
    }

    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        let sent = unsafe { shout_send(self.shout, data.as_ptr(), data.len()) };
        if sent != 0 {
            return Err(self.error());
        }
        unsafe {
            shout_sync(self.shout);
        }
        Ok(())
    }

    fn set_song_title(&mut self, title: &str) -> Result<(), String> {
        let metadata = unsafe { shout_metadata_new() };
        if metadata.is_null() {
            return Err(String::from("shout_metadata_new returned null"));
        }

        let name = CString::new("song").map_err(|_| String::from("string contains null byte"))?;
        let value = CString::new(title).map_err(|_| String::from("string contains null byte"))?;
        let add_result = unsafe { shout_metadata_add(metadata, name.as_ptr(), value.as_ptr()) };
        if add_result != 0 {
            unsafe {
                shout_metadata_free(metadata);
            }
            return Err(self.error());
        }

        let set_result = unsafe { shout_set_metadata(self.shout, metadata) };
        unsafe {
            shout_metadata_free(metadata);
        }
        if set_result == 0 {
            Ok(())
        } else {
            Err(self.error())
        }
    }

    fn set_string(
        &self,
        setter: unsafe extern "C" fn(*mut Shout, *const c_char) -> c_int,
        value: &str,
    ) -> Result<(), String> {
        let value = CString::new(value).map_err(|_| String::from("string contains null byte"))?;
        let result = unsafe { setter(self.shout, value.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(self.error())
        }
    }

    fn set_int(
        &self,
        setter: unsafe extern "C" fn(*mut Shout, c_int) -> c_int,
        value: c_int,
    ) -> Result<(), String> {
        let result = unsafe { setter(self.shout, value) };
        if result == 0 {
            Ok(())
        } else {
            Err(self.error())
        }
    }

    fn error(&self) -> String {
        let ptr = unsafe { shout_get_error(self.shout) };
        if ptr.is_null() {
            return String::from("unknown libshout error");
        }
        unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

impl Drop for LibshoutClient {
    fn drop(&mut self) {
        unsafe {
            let _ = shout_close(self.shout);
            shout_free(self.shout);
        }
    }
}

struct HttpIcecastClient {
    stream: TcpStream,
    host: String,
    port: u16,
    mountpoint: String,
    password: String,
}

impl HttpIcecastClient {
    fn connect(config: &StreamingConfig) -> Result<Self, String> {
        let host = config.host.trim().to_string();
        let port = config.port.clamp(1, 65_535) as u16;
        let mountpoint = normalized_mountpoint(&config.mountpoint);
        let password = config.password.clone();
        let mut last_error = String::new();
        for (method, version) in [("PUT", "HTTP/1.1"), ("SOURCE", "HTTP/1.0")] {
            match open_source_stream(config, &host, port, &mountpoint, &password, method, version) {
                Ok(stream) => {
                    return Ok(Self {
                        stream,
                        host,
                        port,
                        mountpoint,
                        password,
                    });
                }
                Err(error) => last_error = error,
            }
        }

        Err(last_error)
    }

    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.stream
            .write_all(data)
            .map_err(|error| format!("AAC source send failed: {error}"))
    }

    fn set_song_title(&mut self, title: &str) -> Result<(), String> {
        let path = format!(
            "/admin/metadata?mount={}&mode=updinfo&song={}",
            percent_encode(&self.mountpoint),
            percent_encode(title)
        );
        let mut stream = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|error| format!("AAC metadata connect failed: {error}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|error| format!("AAC metadata read timeout failed: {error}"))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|error| format!("AAC metadata write timeout failed: {error}"))?;
        let request = format!(
            "GET {path} HTTP/1.1\r\n\
             Host: {}:{}\r\n\
             User-Agent: OpenStudio\r\n\
             Authorization: Basic {}\r\n\
             Connection: close\r\n\
             \r\n",
            self.host,
            self.port,
            basic_auth("source", &self.password),
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| format!("AAC metadata send failed: {error}"))?;
        Ok(())
    }
}

fn open_source_stream(
    config: &StreamingConfig,
    host: &str,
    port: u16,
    mountpoint: &str,
    password: &str,
    method: &str,
    version: &str,
) -> Result<TcpStream, String> {
    let mut stream = TcpStream::connect((host, port))
        .map_err(|error| format!("AAC source connect failed: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|error| format!("AAC source read timeout failed: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|error| format!("AAC source write timeout failed: {error}"))?;
    let host_header = if port == 80 {
        host.to_string()
    } else {
        format!("{host}:{port}")
    };
    let request = format!(
        "{method} {mountpoint} {version}\r\n\
         Authorization: Basic {}\r\n\
         Host: {host_header}\r\n\
         User-Agent: OpenStudio\r\n\
         Content-Type: audio/aac\r\n\
         ice-bitrate: {}\r\n\
         ice-audio-info: ice-bitrate={};ice-samplerate={};ice-channels={}\r\n\
         \r\n",
        basic_auth("source", password),
        config.bitrate_kbps.clamp(8, 320),
        config.bitrate_kbps.clamp(8, 320),
        config.sample_rate.clamp(8_000, 48_000),
        config.channels.clamp(1, 2),
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("AAC source handshake failed: {error}"))?;
    read_success_response(&mut stream).map_err(|error| format!("AAC source rejected: {error}"))?;
    Ok(stream)
}

fn read_success_response(stream: &mut TcpStream) -> Result<(), String> {
    let mut response = Vec::with_capacity(512);
    let mut byte = [0_u8; 1];
    while response.len() < 4096 {
        let read = stream
            .read(&mut byte)
            .map_err(|error| format!("response read failed: {error}"))?;
        if read == 0 {
            break;
        }
        response.push(byte[0]);
        if response.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let response_text = String::from_utf8_lossy(&response);
    let first_line = response_text.lines().next().unwrap_or("empty response");
    if first_line.contains(" 200 ") || first_line.ends_with(" 200 OK") {
        Ok(())
    } else {
        Err(first_line.to_string())
    }
}

fn normalized_mountpoint(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('/') {
        value.to_string()
    } else {
        format!("/{value}")
    }
}

fn basic_auth(user: &str, password: &str) -> String {
    base64_encode(format!("{user}:{password}").as_bytes())
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}
