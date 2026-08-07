//! Try-before-you-commit installs.
//!
//! `install_temp` drops a Direct-kind app into `~/Downloads/bellwether-temp/<id>/`
//! instead of installing it properly — good for kicking the tires on
//! something you just built, without cluttering `~/.local/bin` or an app
//! launcher's search until you're sure you want to keep it. Anything left
//! there past the TTL (default 48h) gets swept on the next `bellwether`
//! invocation via `cleanup_expired`.
//!
//! `promote` turns a temp install into a proper one: moves it into
//! `~/.local/bin` and writes a `.desktop` launcher entry, then deletes the
//! temp copy — there's nothing left to expire.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{BwError, BwResult};
use crate::model::{AppDef, DirectKind};

const MARKER_FILE: &str = ".bellwether-installed-at";
const DEFAULT_TTL_HOURS: u64 = 48;

fn home_dir() -> BwResult<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| BwError::Other("could not determine $HOME".into()))
}

/// Where temp installs live: ~/Downloads/bellwether-temp/
pub fn temp_root() -> BwResult<PathBuf> {
    Ok(home_dir()?.join("Downloads").join("bellwether-temp"))
}

fn app_temp_dir(app: &AppDef) -> BwResult<PathBuf> {
    Ok(temp_root()?.join(app.id))
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn ttl_hours() -> u64 {
    std::env::var("BELLWETHER_TEMP_TTL_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TTL_HOURS)
}

fn run_ok(cmd: &mut Command) -> BwResult<()> {
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(BwError::CommandFailed {
            cmd: format!("{cmd:?}"),
            status: status.code(),
        })
    }
}

/// Downloads a Direct-kind app into the temp holding area. No root
/// required — everything stays under $HOME. Errors clearly if the app
/// doesn't have a `direct` spec, since temp mode only makes sense for
/// standalone-binary installs (AppImage/tarball/plain executable), not
/// apt/pacman/flatpak packages that register with the system.
pub fn install_temp(app: &AppDef) -> BwResult<PathBuf> {
    let d = app.install.direct.ok_or_else(|| {
        BwError::Other(format!(
            "'{}' has no direct-download spec — temp installs only work for \
             apps with a Direct install method (typically your own apps in \
             my_apps.rs)",
            app.id
        ))
    })?;

    let dir = app_temp_dir(app)?;
    fs::create_dir_all(&dir)?;

    let dest = dir.join(d.install_name);
    run_ok(
        Command::new("curl")
            .args(["-L", "--fail", "-o"])
            .arg(&dest)
            .arg(d.url),
    )?;

    match d.kind {
        DirectKind::AppImage => run_ok(Command::new("chmod").arg("+x").arg(&dest))?,
        DirectKind::TarGz => run_ok(
            Command::new("tar")
                .args(["xzf"])
                .arg(&dest)
                .args(["-C"])
                .arg(&dir),
        )?,
        DirectKind::Deb => {
            // Deliberately not `dpkg -i`'d here — temp mode is for
            // standalone binaries you can just run, not packages that
            // need registering with the system package database. Once
            // you're sure you want it, install it properly instead.
        }
    }

    let marker = dir.join(MARKER_FILE);
    let mut f = fs::File::create(marker)?;
    write!(f, "{}", now_epoch())?;

    Ok(dest)
}

/// Sweeps every temp install older than the TTL (default 48h, override
/// with $BELLWETHER_TEMP_TTL_HOURS). Returns the ids that got cleared.
/// Safe to call on every invocation — cheap, and does nothing if the temp
/// directory doesn't exist or nothing's expired yet.
pub fn cleanup_expired() -> Vec<String> {
    let mut cleared = Vec::new();
    let Ok(root) = temp_root() else {
        return cleared;
    };
    let Ok(entries) = fs::read_dir(root) else {
        return cleared;
    };
    let ttl_secs = ttl_hours() * 3600;
    let now = now_epoch();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(contents) = fs::read_to_string(path.join(MARKER_FILE)) else {
            continue;
        };
        let Ok(installed_at) = contents.trim().parse::<u64>() else {
            continue;
        };
        if now.saturating_sub(installed_at) >= ttl_secs {
            let id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            if fs::remove_dir_all(&path).is_ok() {
                cleared.push(id);
            }
        }
    }
    cleared
}

/// Promotes a temp install to a proper one: copies the binary into
/// ~/.local/bin, makes it executable, writes a .desktop launcher entry
/// (so a graphical app search picks it up too), then deletes the temp
/// copy — there's nothing left for `cleanup_expired` to sweep.
///
/// `is_gui` controls the .desktop entry's Terminal field: false (the
/// default assumption) launches it inside a terminal, since bellwether's
/// own ecosystem is mostly CLI/TUI tools.
pub fn promote(app: &AppDef, is_gui: bool) -> BwResult<PathBuf> {
    let d = app
        .install
        .direct
        .ok_or_else(|| BwError::Other(format!("'{}' has no direct-download spec", app.id)))?;

    let dir = app_temp_dir(app)?;
    let src = dir.join(d.install_name);
    if !src.exists() {
        return Err(BwError::Other(format!(
            "no temp install found for '{}' — run `bellwether temp {}` first",
            app.id, app.id
        )));
    }

    let home = home_dir()?;
    let bin_dir = home.join(".local").join("bin");
    fs::create_dir_all(&bin_dir)?;
    let dest = bin_dir.join(d.install_name);
    fs::copy(&src, &dest)?;
    run_ok(Command::new("chmod").arg("+x").arg(&dest))?;

    let apps_dir = home.join(".local").join("share").join("applications");
    fs::create_dir_all(&apps_dir)?;
    let desktop_path = apps_dir.join(format!("{}.desktop", app.id));
    let terminal = if is_gui { "false" } else { "true" };
    let contents = format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment={}\nExec={}\nTerminal={}\nCategories=Utility;\n",
        app.name,
        app.description,
        dest.display(),
        terminal
    );
    fs::write(desktop_path, contents)?;

    fs::remove_dir_all(&dir)?;
    Ok(dest)
}
