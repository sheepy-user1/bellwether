mod cli;
mod commands;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
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
        Some(Commands::Tui) | None => tui::run()?,
    }

    Ok(())
}
