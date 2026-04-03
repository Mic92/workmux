//! Cross-process lock for serializing `.git/config` writes.
//!
//! `git config` acquires `config.lock` with `O_EXCL` and no retry, so
//! concurrent workmux processes racing to create worktrees in the same repo
//! fail with "could not lock config file". We hold an advisory flock on a
//! sidecar file around the config-writing git calls to serialize them.

use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

#[cfg(unix)]
use nix::fcntl::{Flock, FlockArg};

/// RAII guard. Lock released on drop.
#[cfg(unix)]
pub type ConfigLock = Flock<File>;
#[cfg(not(unix))]
pub struct ConfigLock(File);

/// Block until we hold an exclusive lock on `<git_common_dir>/workmux.lock`.
/// Pass the git-common-dir so all worktrees of one repo share the same lock.
pub fn lock_config_writes(git_common_dir: &Path) -> Result<ConfigLock> {
    let lock_path = git_common_dir.join("workmux.lock");
    let file = File::create(&lock_path)
        .with_context(|| format!("Failed to create lock file {}", lock_path.display()))?;

    #[cfg(unix)]
    return Flock::lock(file, FlockArg::LockExclusive)
        .map_err(|(_, e)| e)
        .with_context(|| format!("flock({}) failed", lock_path.display()));

    // No-op on non-Unix; the race hasn't been reported there.
    #[cfg(not(unix))]
    return Ok(ConfigLock(file));
}
