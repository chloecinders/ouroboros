use std::io;
use std::path::Path;
use std::time::Duration;

use crate::app::updater::Handoff;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("the artifact is not a readable zip")]
    Unreadable,
    #[error("the artifact contains no file named {0}")]
    Missing(String),
}

pub fn unpack(archive: Vec<u8>, binary: &str) -> Result<Vec<u8>, Failure> {
    let mut zip =
        zip::ZipArchive::new(io::Cursor::new(archive)).map_err(|_| Failure::Unreadable)?;

    for index in 0..zip.len() {
        let Ok(mut entry) = zip.by_index(index) else {
            continue;
        };

        let name = String::from(entry.name());

        if name.ends_with('/') || name.rsplit('/').next() != Some(binary) {
            continue;
        }

        let mut buffer = Vec::with_capacity(entry.size() as usize);

        io::copy(&mut entry, &mut buffer).map_err(|_| Failure::Unreadable)?;

        return Ok(buffer);
    }

    Err(Failure::Missing(String::from(binary)))
}

pub async fn stage(binary: &[u8], path: &Path) -> io::Result<()> {
    tokio::fs::write(path, binary).await?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).await?;
    }

    Ok(())
}

pub fn install(from: &Path, to: &Path) -> io::Result<()> {
    for attempt in 1u64..25 {
        if replace(from, to).is_ok() {
            return Ok(());
        }

        std::thread::sleep(Duration::from_millis(100 * attempt.min(20)));
    }

    replace(from, to)
}

fn replace(from: &Path, to: &Path) -> io::Result<()> {
    if to.exists() {
        std::fs::remove_file(to)?;
    }

    std::fs::copy(from, to)?;

    Ok(())
}

pub fn launch(binary: &Path, argument: &str) -> io::Result<()> {
    std::process::Command::new(binary).arg(argument).spawn()?;

    Ok(())
}

pub fn commit(staged: &Path, handoff: &Handoff) -> io::Result<()> {
    match cfg!(windows) {
        true => launch(staged, &handoff.flag("--update")),
        false => {
            std::fs::write("update.txt", handoff.encode())?;

            install(staged, Path::new(&super::binary_name()))
        }
    }
}

pub fn relay(handoff: &Handoff) -> io::Result<()> {
    let staged = std::env::current_exe()?;
    let target = staged
        .parent()
        .unwrap_or(Path::new("."))
        .join(super::binary_name());

    install(&staged, &target)?;

    tracing::info!("installed {}, handing back", target.display());

    launch(&target, &handoff.flag("--id"))
}
