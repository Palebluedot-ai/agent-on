//! Private atomic files and short process-scoped locks shared by delivery safeguards.
use chrono::Utc;
use serde::Serialize;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
type Result<T> = std::result::Result<T, String>;
static SERIAL: AtomicU64 = AtomicU64::new(0);
pub(crate) fn lock_path(path: &Path) -> Result<Lock> {
    let parent = path.parent().ok_or("lock path has no parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    match fs::symlink_metadata(path) {
        Ok(m) if !m.is_file() => {
            return Err(format!("lock is not a regular file: {}", path.display()))
        }
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.to_string()),
    }
    let mut options = OpenOptions::new();
    options.write(true).read(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path).map_err(|e| e.to_string())?;
    // Recheck after open and never truncate/write a lock file. A raced symlink
    // or a hardlink must not damage its target merely to acquire a lease.
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("lock path changed to a non-regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let opened = file.metadata().map_err(|e| e.to_string())?;
        if (meta.dev(), meta.ino()) != (opened.dev(), opened.ino()) {
            return Err("lock path changed during open".into());
        }
    }
    for _ in 0..40 {
        match file.try_lock() {
            Ok(()) => {
                return Ok(Lock(file));
            }
            Err(TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(10)),
            Err(TryLockError::Error(e)) => return Err(e.to_string()),
        }
    }
    Err(format!(
        "shared writer busy or owner unknown: {}",
        path.display()
    ))
}

pub(crate) struct Lock(File);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

pub(crate) fn now() -> String {
    Utc::now().to_rfc3339()
}
fn nonce() -> String {
    format!(
        "{}-{}-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default(),
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    )
}
pub(crate) fn write_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    write_private(
        path,
        &serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?,
    )
}
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("missing parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(".tmp-{}", nonce()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp).map_err(|e| e.to_string())?;
    let outcome = file
        .write_all(bytes)
        .and_then(|_| file.sync_all())
        .and_then(|_| fs::rename(&temp, path));
    if outcome.is_err() {
        let _ = fs::remove_file(&temp);
    }
    outcome.map_err(|e| e.to_string())
}
