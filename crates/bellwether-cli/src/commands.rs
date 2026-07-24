use anyhow::Result;
use bellwether_core::model::Category;
use bellwether_core::SystemInfo;
use bellwether_core::{catalog, installer};

pub fn list() {
    let categories = [
        Category::Monitoring,
        Category::Power,
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

pub fn install(ids: &[String], all: bool) -> Result<()> {
    let sys = SystemInfo::detect();

    let targets: Vec<_> = if all {
        catalog()
    } else {
        let mut found = Vec::new();
        for id in ids {
            match bellwether_core::find(id) {
                Some(app) => found.push(app),
                None => {
                    eprintln!(
                        "warning: no app with id '{id}' in the catalog (see `bellwether list`)"
                    );
                }
            }
        }
        found
    };

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
