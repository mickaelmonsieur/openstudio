use std::ffi::{c_char, c_int, c_uchar, c_void, CString};

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
    shout: *mut Shout,
}

impl IcecastClient {
    pub fn connect(config: &StreamingConfig) -> Result<Self, String> {
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

    pub fn send(&mut self, data: &[u8]) -> Result<(), String> {
        let sent = unsafe { shout_send(self.shout, data.as_ptr(), data.len()) };
        if sent != 0 {
            return Err(self.error());
        }
        unsafe {
            shout_sync(self.shout);
        }
        Ok(())
    }

    pub fn set_song_title(&mut self, title: &str) -> Result<(), String> {
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

impl Drop for IcecastClient {
    fn drop(&mut self) {
        unsafe {
            let _ = shout_close(self.shout);
            shout_free(self.shout);
        }
    }
}
