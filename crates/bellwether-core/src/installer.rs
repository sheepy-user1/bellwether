use std::process::{Command, Stdio};

use crate::error::{BwError, BwResult};
use crate::model::{AppDef, DirectKind, InstallMethod};
use crate::sysinfo::SystemInfo;

/// Default global fallback order used when an app doesn't specify its own
/// `preference` list.
const DEFAULT_PREFERENCE: &[InstallMethod] = &[
    InstallMethod::Native,
    InstallMethod::Aur,
    InstallMethod::Flatpak,
    InstallMethod::Direct,
];

/// Reports what happened for one app, so the CLI/TUI can print a summary.
#[derive(Debug)]
pub struct InstallOutcome {
    pub app_id: &'static str,
    pub method_used: Option<InstallMethod>,
    pub post_install_notes: Vec<String>,
}

fn run(cmd: &mut Command) -> BwResult<()> {
    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(BwError::CommandFailed {
            cmd: format!("{cmd:?}"),
            status: status.code(),
        })
    }
}

fn sudo_wrap(sys: &SystemInfo, program: &str, args: &[&str]) -> Command {
    if sys.is_root {
        let mut c = Command::new(program);
        c.args(args);
        c
    } else {
        let mut c = Command::new("sudo");
        c.arg(program).args(args);
        c
    }
}

/// Picks the first install method that is both defined for the app *and*
/// available on this system, walking the app's preference list (or the
/// global default if the app doesn't set one).
pub fn choose_method(app: &AppDef, sys: &SystemInfo) -> Option<InstallMethod> {
    let prefs = if app.install.preference.is_empty() {
        DEFAULT_PREFERENCE
    } else {
        app.install.preference
    };

    for method in prefs {
        let defined = match method {
            InstallMethod::Native => {
                (sys.has_apt && app.install.apt.is_some())
                    || (sys.has_pacman && app.install.pacman.is_some())
                    || (sys.has_dnf && app.install.dnf.is_some())
            }
            InstallMethod::Aur => sys.aur_helper.is_some() && app.install.aur.is_some(),
            InstallMethod::Flatpak => sys.has_flatpak && app.install.flatpak.is_some(),
            InstallMethod::Direct => app.install.direct.is_some(),
        };
        if defined {
            return Some(*method);
        }
    }
    None
}

fn install_native(app: &AppDef, sys: &SystemInfo) -> BwResult<()> {
    if sys.has_apt {
        if let Some(pkg) = app.install.apt {
            run(&mut sudo_wrap(sys, "apt-get", &["install", "-y", pkg]))?;
            return Ok(());
        }
    }
    if sys.has_pacman {
        if let Some(pkg) = app.install.pacman {
            run(&mut sudo_wrap(
                sys,
                "pacman",
                &["-S", "--noconfirm", "--needed", pkg],
            ))?;
            return Ok(());
        }
    }
    if sys.has_dnf {
        if let Some(pkg) = app.install.dnf {
            run(&mut sudo_wrap(sys, "dnf", &["install", "-y", pkg]))?;
            return Ok(());
        }
    }
    Err(BwError::NoInstallMethod { app: app.id.into() })
}

fn install_aur(app: &AppDef, sys: &SystemInfo) -> BwResult<()> {
    let helper = sys
        .aur_helper
        .ok_or_else(|| BwError::NoInstallMethod { app: app.id.into() })?;
    let pkg = app
        .install
        .aur
        .ok_or_else(|| BwError::NoInstallMethod { app: app.id.into() })?;
    // AUR helpers should NOT be run as root.
    if sys.is_root {
        return Err(BwError::Other(format!(
            "{helper} refuses to run as root; re-run bellwether as a normal user for AUR installs"
        )));
    }
    run(Command::new(helper).args(["-S", "--noconfirm", pkg]))
}

fn install_flatpak(app: &AppDef, _sys: &SystemInfo) -> BwResult<()> {
    let id = app
        .install
        .flatpak
        .ok_or_else(|| BwError::NoInstallMethod { app: app.id.into() })?;
    run(Command::new("flatpak").args(["install", "-y", "--noninteractive", "flathub", id]))
}

fn install_direct(app: &AppDef, sys: &SystemInfo) -> BwResult<()> {
    let d = app
        .install
        .direct
        .ok_or_else(|| BwError::NoInstallMethod { app: app.id.into() })?;
    let tmp = std::env::temp_dir().join(d.install_name);

    run(Command::new("curl").args(["-L", "--fail", "-o", tmp.to_str().unwrap(), d.url]))?;

    match d.kind {
        DirectKind::Deb => {
            run(&mut sudo_wrap(
                sys,
                "apt-get",
                &["install", "-y", tmp.to_str().unwrap()],
            ))?;
        }
        DirectKind::AppImage => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            let dest_dir = format!("{home}/.local/bin");
            std::fs::create_dir_all(&dest_dir)?;
            let dest = format!("{dest_dir}/{}", d.install_name);
            std::fs::copy(&tmp, &dest)?;
            run(Command::new("chmod").args(["+x", &dest]))?;
        }
        DirectKind::TarGz => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            let dest_dir = format!("{home}/.local/opt/{}", d.install_name);
            std::fs::create_dir_all(&dest_dir)?;
            run(Command::new("tar").args(["xzf", tmp.to_str().unwrap(), "-C", &dest_dir]))?;
        }
    }
    Ok(())
}

/// Installs the package portion of an app (not post-install config).
pub fn install_package(app: &AppDef, sys: &SystemInfo, method: InstallMethod) -> BwResult<()> {
    match method {
        InstallMethod::Native => install_native(app, sys),
        InstallMethod::Aur => install_aur(app, sys),
        InstallMethod::Flatpak => install_flatpak(app, sys),
        InstallMethod::Direct => install_direct(app, sys),
    }
}

/// Installs an app end-to-end: picks a method, installs the package, then
/// applies its post-install configuration steps.
pub fn install_app(app: &AppDef, sys: &SystemInfo) -> BwResult<InstallOutcome> {
    let method =
        choose_method(app, sys).ok_or_else(|| BwError::NoInstallMethod { app: app.id.into() })?;
    install_package(app, sys, method)?;
    let notes = crate::configure::apply_post_install(app, sys)?;
    Ok(InstallOutcome {
        app_id: app.id,
        method_used: Some(method),
        post_install_notes: notes,
    })
}

/// Best-effort check for whether an app is already installed. For Flatpak
/// apps we ask flatpak directly; otherwise we look for the app's binary
/// on PATH. This is a heuristic, not a guarantee — a binary of the same
/// name from somewhere else would register as a false positive.
pub fn is_installed(app: &AppDef, sys: &SystemInfo) -> bool {
    if let Some(flatpak_id) = app.install.flatpak {
        if sys.has_flatpak {
            if let Ok(output) = Command::new("flatpak")
                .args(["list", "--app", "--columns=application"])
                .output()
            {
                let out = String::from_utf8_lossy(&output.stdout);
                if out.lines().any(|l| l.trim() == flatpak_id) {
                    return true;
                }
            }
        }
    }
    which::which(app.bin_name()).is_ok()
}

/// Removes an app's package via whichever manager is present and defined
/// for it, preferring native > flatpak > deleting a direct-installed file.
/// Does not attempt to reverse post-install config changes (config files,
/// systemd units) — those are left in place intentionally, so a later
/// reinstall doesn't lose your settings.
pub fn uninstall_app(app: &AppDef, sys: &SystemInfo) -> BwResult<()> {
    if sys.has_apt {
        if let Some(pkg) = app.install.apt {
            return run(&mut sudo_wrap(sys, "apt-get", &["remove", "-y", pkg]));
        }
    }
    if sys.has_pacman {
        if let Some(pkg) = app.install.pacman.or(app.install.aur) {
            return run(&mut sudo_wrap(sys, "pacman", &["-R", "--noconfirm", pkg]));
        }
    }
    if sys.has_dnf {
        if let Some(pkg) = app.install.dnf {
            return run(&mut sudo_wrap(sys, "dnf", &["remove", "-y", pkg]));
        }
    }
    if sys.has_flatpak {
        if let Some(id) = app.install.flatpak {
            return run(Command::new("flatpak").args(["uninstall", "-y", id]));
        }
    }
    if let Some(d) = app.install.direct {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        match d.kind {
            DirectKind::AppImage => {
                let path = format!("{home}/.local/bin/{}", d.install_name);
                std::fs::remove_file(path)?;
                return Ok(());
            }
            DirectKind::TarGz => {
                let path = format!("{home}/.local/opt/{}", d.install_name);
                std::fs::remove_dir_all(path)?;
                return Ok(());
            }
            DirectKind::Deb => {
                return Err(BwError::Other(
                    "installed from a .deb — remove it with your normal package manager instead"
                        .into(),
                ));
            }
        }
    }
    Err(BwError::NoInstallMethod { app: app.id.into() })
}

/// Re-runs the install step (harmless if already installed — most package
/// managers just confirm it's current) and force-reapplies every
/// post-install config step, overwriting any drift back to bellwether's
/// defaults. Use this when something's misbehaving rather than actually
/// missing.
pub fn repair_app(app: &AppDef, sys: &SystemInfo) -> BwResult<InstallOutcome> {
    let method =
        choose_method(app, sys).ok_or_else(|| BwError::NoInstallMethod { app: app.id.into() })?;
    install_package(app, sys, method)?;
    let notes = crate::configure::apply_post_install_forced(app, sys)?;
    Ok(InstallOutcome {
        app_id: app.id,
        method_used: Some(method),
        post_install_notes: notes,
    })
}
