use anyhow::Result;
use bellwether_core::model::{AppDef, Category};
use bellwether_core::SystemInfo;
use bellwether_core::{catalog, installer};

pub fn list() {
    let categories = [
        Category::Monitoring,
        Category::Power,
        Category::Server,
        Category::Utilities,
        Category::Creative,
        Category::Gaming,
        Category::Browser,
        Category::Custom,
    ];
    let all_apps = catalog();
    for cat in categories {
        let apps: Vec<_> = all_apps
            .iter()
            .filter(|a| a.category as u8 == cat as u8)
            .collect();
        if apps.is_empty() {
            continue;
        }
        println!("\n{}", cat.label());
        println!("{}", "-".repeat(cat.label().len()));
        for app in apps {
            println!("  {:<16} {}", app.id, app.description);
        }
    }
    println!("\nInstall with: bellwether install <id> [<id> ...]");
}

pub fn doctor() {
    let sys = SystemInfo::detect();
    println!("System summary");
    println!("--------------");
    println!("package managers : {}", sys.distro_summary());
    println!(
        "AUR helper        : {}",
        sys.aur_helper.unwrap_or("none found")
    );
    println!("running as root   : {}", sys.is_root);
    println!();
    println!("Per-app viability:");
    for app in catalog() {
        let verdict = match installer::choose_method(app, &sys) {
            Some(m) => format!("OK, via {}", m.label()),
            None => "no viable install method on this system".to_string(),
        };
        println!("  {:<16} {}", app.id, verdict);
    }
}

pub fn scan() {
    let sys = SystemInfo::detect();
    println!("Scanning system ({})...\n", sys.distro_summary());
    let mut installed_count = 0;
    for app in catalog() {
        let status = if installer::is_installed(app, &sys) {
            installed_count += 1;
            "installed"
        } else {
            "not installed"
        };
        println!("  {:<16} {}", app.id, status);
    }
    println!(
        "\n{installed_count} app(s) installed out of {} in the catalog.",
        catalog().len()
    );
    println!("Use `bellwether repair <id>` to fix a misbehaving install, or `bellwether remove <id>` to take it off.");
}

pub fn profiles() {
    println!("Available profiles:\n");
    for p in bellwether_core::PROFILES {
        println!("  {:<10} {}", p.id, p.name);
        println!("             {}", p.description);
        println!("             apps: {}\n", p.app_ids.join(", "));
    }
    println!("Use with: bellwether install --profile <id>");
}

/// Resolves ids/--all/--profile into concrete AppDefs, warning about
/// unknown ids or profile names. Exactly one of `all` or `profile` should
/// be meaningful at a time; if `profile` is set it takes priority.
fn resolve_targets(ids: &[String], all: bool, profile: Option<&str>) -> Vec<&'static AppDef> {
    if let Some(pid) = profile {
        return match bellwether_core::find_profile(pid) {
            Some(p) => bellwether_core::profile_apps(p),
            None => {
                eprintln!("warning: no profile named '{pid}' (see `bellwether profiles`)");
                Vec::new()
            }
        };
    }
    if all {
        return catalog();
    }
    let mut found = Vec::new();
    for id in ids {
        match bellwether_core::find(id) {
            Some(app) => found.push(app),
            None => {
                eprintln!("warning: no app with id '{id}' in the catalog (see `bellwether list`)");
            }
        }
    }
    found
}

/// Same as `resolve_targets`, but --all / --profile only pick up apps that
/// are actually installed right now — you don't want `remove --all` or
/// `repair --profile server` blindly touching things you never installed.
fn resolve_installed_targets(
    ids: &[String],
    all: bool,
    profile: Option<&str>,
    sys: &SystemInfo,
) -> Vec<&'static AppDef> {
    if all || profile.is_some() {
        return resolve_targets(ids, all, profile)
            .into_iter()
            .filter(|a| installer::is_installed(a, sys))
            .collect();
    }
    resolve_targets(ids, false, None)
}

pub fn install(ids: &[String], all: bool, profile: Option<&str>) -> Result<()> {
    let sys = SystemInfo::detect();
    let targets = resolve_targets(ids, all, profile);

    if targets.is_empty() {
        eprintln!("nothing to install. Try `bellwether list` to see available app ids.");
        return Ok(());
    }

    let mut failures = Vec::new();
    for app in targets {
        println!("\n==> {} ({})", app.name, app.id);
        match installer::install_app(app, &sys) {
            Ok(outcome) => {
                if let Some(m) = outcome.method_used {
                    println!("    installed via {}", m.label());
                }
                for note in outcome.post_install_notes {
                    println!("    - {note}");
                }
            }
            Err(e) => {
                eprintln!("    FAILED: {e}");
                failures.push(app.id);
            }
        }
    }

    if !failures.is_empty() {
        eprintln!(
            "\n{} app(s) failed: {}",
            failures.len(),
            failures.join(", ")
        );
        std::process::exit(1);
    }
    println!("\nAll done.");
    Ok(())
}

pub fn remove(ids: &[String], all: bool, profile: Option<&str>) -> Result<()> {
    let sys = SystemInfo::detect();
    let targets = resolve_installed_targets(ids, all, profile, &sys);

    if targets.is_empty() {
        eprintln!("nothing to remove. Try `bellwether scan` to see what's installed.");
        return Ok(());
    }

    let mut failures = Vec::new();
    for app in targets {
        println!("\n==> removing {} ({})", app.name, app.id);
        match installer::uninstall_app(app, &sys) {
            Ok(()) => println!("    removed"),
            Err(e) => {
                eprintln!("    FAILED: {e}");
                failures.push(app.id);
            }
        }
    }

    if !failures.is_empty() {
        eprintln!(
            "\n{} app(s) failed to remove: {}",
            failures.len(),
            failures.join(", ")
        );
        std::process::exit(1);
    }
    println!("\nAll done.");
    Ok(())
}

pub fn repair(ids: &[String], all: bool, profile: Option<&str>) -> Result<()> {
    let sys = SystemInfo::detect();
    let targets = resolve_installed_targets(ids, all, profile, &sys);

    if targets.is_empty() {
        eprintln!("nothing to repair. Try `bellwether scan` to see what's installed.");
        return Ok(());
    }

    let mut failures = Vec::new();
    for app in targets {
        println!("\n==> repairing {} ({})", app.name, app.id);
        match installer::repair_app(app, &sys) {
            Ok(outcome) => {
                if let Some(m) = outcome.method_used {
                    println!("    reinstalled via {}", m.label());
                }
                for note in outcome.post_install_notes {
                    println!("    - {note}");
                }
            }
            Err(e) => {
                eprintln!("    FAILED: {e}");
                failures.push(app.id);
            }
        }
    }

    if !failures.is_empty() {
        eprintln!(
            "\n{} app(s) failed to repair: {}",
            failures.len(),
            failures.join(", ")
        );
        std::process::exit(1);
    }
    println!("\nAll done.");
    Ok(())
}
