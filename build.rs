fn main() {
    println!("cargo:rustc-check-cfg=cfg(openstudio_has_lame)");
    println!("cargo:rustc-check-cfg=cfg(openstudio_has_shout)");

    if let Some(lame_lib_dir) = std::env::var("OPENSTUDIO_LAME_LIB_DIR")
        .ok()
        .filter(|path| std::path::Path::new(path).is_dir())
    {
        println!("cargo:rustc-cfg=openstudio_has_lame");
        println!("cargo:rustc-link-search=native={lame_lib_dir}");
        println!("cargo:rustc-link-lib=mp3lame");
    } else if pkg_config_has("lame") {
        println!("cargo:rustc-cfg=openstudio_has_lame");
        emit_pkg_config_links("lame");
    } else if let Some(lame_lib_dir) = existing_dir(&[
        "/opt/homebrew/opt/lame/lib",
        "/usr/local/opt/lame/lib",
        "/usr/lib",
        "/usr/local/lib",
    ]) {
        println!("cargo:rustc-cfg=openstudio_has_lame");
        println!("cargo:rustc-link-search=native={lame_lib_dir}");
        println!("cargo:rustc-link-lib=mp3lame");
    }
    if let Some(shout_lib_dir) = std::env::var("OPENSTUDIO_SHOUT_LIB_DIR")
        .ok()
        .filter(|path| std::path::Path::new(path).is_dir())
    {
        println!("cargo:rustc-cfg=openstudio_has_shout");
        println!("cargo:rustc-link-search=native={shout_lib_dir}");
        println!("cargo:rustc-link-lib=shout");
    } else if pkg_config_has("shout") {
        println!("cargo:rustc-cfg=openstudio_has_shout");
        emit_pkg_config_links("shout");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/icon.ico");
        res.compile().expect("failed to embed Windows resources");
    }
}

fn existing_dir(candidates: &[&str]) -> Option<String> {
    candidates
        .iter()
        .copied()
        .find(|path| std::path::Path::new(path).is_dir())
        .map(String::from)
}

fn pkg_config_has(package: &str) -> bool {
    std::process::Command::new("pkg-config")
        .args(["--exists", package])
        .status()
        .is_ok_and(|status| status.success())
}

fn emit_pkg_config_links(package: &str) {
    let Ok(output) = std::process::Command::new("pkg-config")
        .args(["--libs", package])
        .output()
    else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let Ok(raw) = String::from_utf8(output.stdout) else {
        return;
    };
    for token in raw.split_whitespace() {
        if let Some(path) = token.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(lib) = token.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={lib}");
        }
    }
}
