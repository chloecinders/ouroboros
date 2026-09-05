pub mod apply;
pub mod download;

use std::path::Path;

use tracing::{error, info, warn};

pub const BINARY: &str = env!("CARGO_PKG_NAME");

pub fn binary_name() -> String {
    match cfg!(windows) {
        true => format!("{BINARY}.exe"),
        false => String::from(BINARY),
    }
}

pub fn staged_name() -> String {
    format!("new_{}", binary_name())
}

pub fn is_stale(name: &str) -> bool {
    let name = name.to_ascii_lowercase();

    name.starts_with("new_") && name.contains(&BINARY.to_ascii_lowercase())
}

pub fn cleanup(directory: &Path) {
    let running = std::env::current_exe().ok().and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    });

    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !path.is_file() || !is_stale(name) || running.as_deref() == Some(name) {
            continue;
        }

        match std::fs::remove_file(&path) {
            Ok(()) => info!("removed the staged build {name}"),
            Err(failure) => warn!("could not remove the staged build {name}; err = {failure}"),
        }
    }
}

pub fn intercept() -> bool {
    if cfg!(windows) && std::env::args().any(|arg| arg == "--update") {
        info!("installing the staged build over {}", binary_name());

        if let Err(failure) = apply::relay() {
            error!("the staged build could not install itself; err = {failure}");
        }

        return true;
    }

    cleanup(Path::new("."));

    false
}
