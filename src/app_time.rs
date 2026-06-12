#[cfg(unix)]
fn system_locale_cstr() -> &'static std::ffi::CString {
    static LOCALE: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();
    LOCALE.get_or_init(|| {
        #[cfg(target_os = "macos")]
        if let Ok(out) = std::process::Command::new("defaults")
            .args(["read", "NSGlobalDomain", "AppleLocale"])
            .output()
        {
            if let Ok(s) = std::str::from_utf8(&out.stdout) {
                let s = s.trim();
                if !s.is_empty() {
                    let locale = if s.contains('.') {
                        s.to_string()
                    } else {
                        format!("{}.UTF-8", s)
                    };
                    if let Ok(cs) = std::ffi::CString::new(locale) {
                        return cs;
                    }
                }
            }
        }
        std::ffi::CString::new("").unwrap()
    })
}

#[cfg(unix)]
pub(crate) fn current_hour() -> String {
    let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return String::from("--:--:--");
    };
    let timestamp = now.as_secs() as libc::time_t;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
    let local = unsafe {
        if libc::localtime_r(&timestamp, local.as_mut_ptr()).is_null() {
            return String::from("--:--:--");
        }
        local.assume_init()
    };
    format!(
        "{:02}:{:02}:{:02}",
        local.tm_hour, local.tm_min, local.tm_sec
    )
}

#[cfg(not(unix))]
pub(crate) fn current_hour() -> String {
    let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return String::from("--:--:--");
    };
    crate::ui::styles::fmt_hms(std::time::Duration::from_secs(
        now.as_secs() % (24 * 60 * 60),
    ))
}

#[cfg(unix)]
pub(crate) fn current_date() -> String {
    let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return String::from("---");
    };
    let timestamp = now.as_secs() as libc::time_t;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
    let local = unsafe {
        if libc::localtime_r(&timestamp, local.as_mut_ptr()).is_null() {
            return String::from("---");
        }
        local.assume_init()
    };
    unsafe {
        libc::setlocale(libc::LC_TIME, system_locale_cstr().as_ptr());
        let mut wday_buf = [0u8; 32];
        libc::strftime(
            wday_buf.as_mut_ptr() as *mut libc::c_char,
            wday_buf.len(),
            b"%A\0".as_ptr() as *const libc::c_char,
            &local,
        );
        let wday_len = wday_buf.iter().position(|&b| b == 0).unwrap_or(0);
        let mut month_buf = [0u8; 32];
        libc::strftime(
            month_buf.as_mut_ptr() as *mut libc::c_char,
            month_buf.len(),
            b"%B\0".as_ptr() as *const libc::c_char,
            &local,
        );
        let month_len = month_buf.iter().position(|&b| b == 0).unwrap_or(0);
        let wday = String::from_utf8_lossy(&wday_buf[..wday_len]).to_uppercase();
        let month = String::from_utf8_lossy(&month_buf[..month_len]).to_uppercase();
        format!(
            "{} {} {} {}",
            wday,
            local.tm_mday,
            month,
            local.tm_year + 1900
        )
    }
}

#[cfg(not(unix))]
pub(crate) fn current_date() -> String {
    String::from("---")
}
