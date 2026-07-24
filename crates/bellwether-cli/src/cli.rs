use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "bellwether",
    version,
    about = "Bellwether: install and configure your Linux apps with sane defaults, from a script, a binary, or a TUI.",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List every app in the catalog, grouped by category.
    List,
    /// Show what bellwether has detected about this machine.
    Doctor,
    /// Install one or more apps by id (see `bellwether list`).
    Install {
        /// App ids to install, e.g. `bellwether install btop steam`
        ids: Vec<String>,
        /// Install every app in the catalog.
        #[arg(long)]
        all: bool,
    },
    /// Launch the interactive terminal UI (checklist + mouse clicks).
    Tui,
}
