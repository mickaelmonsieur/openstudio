use std::path::PathBuf;

pub(crate) fn user_db_config_path() -> Option<PathBuf> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            if !config_home.is_empty() {
                return Some(PathBuf::from(config_home).join("openstudio/database.json"));
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return Some(PathBuf::from(home).join(".config/openstudio/database.json"));
            }
        }
    }

    None
}

pub(crate) fn db_config_path() -> PathBuf {
    if let Some(candidate) = user_db_config_path() {
        if candidate.exists() {
            return candidate;
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        let candidates = [
            // macOS app bundle: Contents/Resources/database.json
            exe.parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("Resources/database.json")),
            // Windows packaged: resources are next to the executable.
            exe.parent().map(|p| p.join("database.json")),
            exe.parent().map(|p| p.join("config/database.json")),
            // Debian packaged: resources are installed under /usr/lib/openstudio.
            Some(PathBuf::from("/usr/lib/openstudio/database.json")),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() {
                return candidate;
            }
        }
    }

    // Dev: config/database.json relative to working directory
    PathBuf::from("config/database.json")
}

pub(crate) fn db_config_save_path() -> PathBuf {
    user_db_config_path().unwrap_or_else(db_config_path)
}

pub(crate) fn migrations_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        // macOS app bundle: Contents/Resources/migrations/
        if let Some(candidate) = exe
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("Resources/migrations"))
        {
            if candidate.exists() {
                return candidate;
            }
        }
        // Windows packaged: migrations/ next to the exe
        if let Some(candidate) = exe.parent().map(|p| p.join("migrations")) {
            if candidate.exists() {
                return candidate;
            }
        }
        // Debian packaged: resources are installed under /usr/lib/openstudio.
        let candidate = PathBuf::from("/usr/lib/openstudio/migrations");
        if candidate.exists() {
            return candidate;
        }
    }
    // Dev: migrations/ relative to working directory
    PathBuf::from("migrations")
}

pub(crate) fn pg_quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[cfg(target_os = "windows")]
pub(crate) const DEFAULT_PSQL_PATH: &str = r"C:\Program Files\PostgreSQL\18\bin\psql.exe";
#[cfg(target_os = "macos")]
pub(crate) const DEFAULT_PSQL_PATH: &str =
    "/Applications/Postgres.app/Contents/Versions/18/bin/psql";
#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) const DEFAULT_PSQL_PATH: &str = "/usr/lib/postgresql/18/bin/psql";
