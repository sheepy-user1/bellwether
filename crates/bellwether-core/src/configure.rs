use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::error::{BwError, BwResult};
use crate::model::{AppDef, PostInstallStep};
use crate::sysinfo::SystemInfo;

fn home_dir() -> BwResult<PathBuf> {
    // When running under sudo, prefer the *invoking* user's home so config
    // files land in the right place instead of /root.
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if !sudo_user.is_empty() && sudo_user != "root" {
            return Ok(PathBuf::from(format!("/home/{sudo_user}")));
        }
    }
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| BwError::Other("could not determine $HOME".into()))
}

fn run_shell(cmd: &str) -> BwResult<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(BwError::CommandFailed {
            cmd: cmd.to_string(),
            status: status.code(),
        })
    }
}

fn run_shell_as_root(sys: &SystemInfo, cmd: &str) -> BwResult<()> {
    if sys.is_root {
        run_shell(cmd)
    } else {
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(BwError::CommandFailed {
                cmd: cmd.to_string(),
                status: status.code(),
            })
        }
    }
}

/// Applies every post-install step for an app, returning human-readable
/// notes describing what happened (for CLI/TUI summaries). Respects each
/// step's own `force` flag for config files (won't clobber your edits).
pub fn apply_post_install(app: &AppDef, sys: &SystemInfo) -> BwResult<Vec<String>> {
    apply_post_install_impl(app, sys, false)
}

/// Same as `apply_post_install`, but treats every `WriteUserFile` step as
/// forced — overwrites existing config back to bellwether's defaults.
/// Used by `repair_app` for "fix this because something's drifted /
/// broken" rather than a fresh install.
pub fn apply_post_install_forced(app: &AppDef, sys: &SystemInfo) -> BwResult<Vec<String>> {
    apply_post_install_impl(app, sys, true)
}

fn apply_post_install_impl(
    app: &AppDef,
    sys: &SystemInfo,
    force_all: bool,
) -> BwResult<Vec<String>> {
    let mut notes = Vec::new();
    let home = home_dir();

    for step in app.post_install {
        match step {
            PostInstallStep::WriteUserFile {
                rel_path,
                content,
                force,
            } => {
                let force = force_all || *force;
                let home = home.as_ref().map_err(|e| BwError::Other(e.to_string()))?;
                let path = home.join(rel_path);
                if path.exists() && !force {
                    notes.push(format!("kept existing config at {}", path.display()));
                    continue;
                }
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut f = std::fs::File::create(&path)?;
                f.write_all(content.as_bytes())?;
                notes.push(format!("wrote config to {}", path.display()));
            }
            PostInstallStep::RootShell(cmd) => {
                run_shell_as_root(sys, cmd)?;
                notes.push(format!("ran (as root): {cmd}"));
            }
            PostInstallStep::UserShell(cmd) => {
                run_shell(cmd)?;
                notes.push(format!("ran: {cmd}"));
            }
            PostInstallStep::EnableService { unit, start_now } => {
                let args: &[&str] = if *start_now {
                    &["enable", "--now"]
                } else {
                    &["enable"]
                };
                let mut full_args: Vec<&str> = args.to_vec();
                full_args.push(unit);
                let cmd_str = format!("systemctl {}", full_args.join(" "));
                run_shell_as_root(sys, &cmd_str)?;
                notes.push(format!("enabled systemd unit: {unit}"));
            }
            PostInstallStep::Note(text) => {
                notes.push(format!("note: {text}"));
            }
        }
    }
    Ok(notes)
}
