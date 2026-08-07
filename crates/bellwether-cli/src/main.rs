mod cli;
mod commands;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cleared = bellwether_core::temp::cleanup_expired();
    if !cleared.is_empty() {
        eprintln!(
            "(cleared out {} aged-out temp install(s): {})",
            cleared.len(),
            cleared.join(", ")
        );
    }

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => commands::list(),
        Some(Commands::Doctor) => commands::doctor(),
        Some(Commands::Scan) => commands::scan(),
        Some(Commands::Profiles) => commands::profiles(),
        Some(Commands::Install { ids, all, profile }) => {
            commands::install(&ids, all, profile.as_deref())?
        }
        Some(Commands::Remove { ids, all, profile }) => {
            commands::remove(&ids, all, profile.as_deref())?
        }
        Some(Commands::Repair { ids, all, profile }) => {
            commands::repair(&ids, all, profile.as_deref())?
        }
        Some(Commands::Temp { ids }) => commands::temp(&ids)?,
        Some(Commands::Promote { ids, gui }) => commands::promote(&ids, gui)?,
        Some(Commands::Tui) | None => tui::run()?,
    }

    Ok(())
}
