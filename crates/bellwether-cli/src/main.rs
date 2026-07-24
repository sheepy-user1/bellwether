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
        Some(Commands::Install { ids, all }) => commands::install(&ids, all)?,
        Some(Commands::Remove { ids, all }) => commands::remove(&ids, all)?,
        Some(Commands::Repair { ids, all }) => commands::repair(&ids, all)?,
        Some(Commands::Tui) | None => tui::run()?,
    }

    Ok(())
}
