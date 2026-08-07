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
    /// Scan the system and report which catalog apps are installed.
    Scan,
    /// List available profiles (curated bundles of apps).
    Profiles,
    /// Install one or more apps by id (see `bellwether list`).
    Install {
        /// App ids to install, e.g. `bellwether install btop steam`
        ids: Vec<String>,
        /// Install every app in the catalog.
        #[arg(long)]
        all: bool,
        /// Install a named profile's apps instead, e.g. `--profile server`
        /// (see `bellwether profiles`).
        #[arg(long)]
        profile: Option<String>,
    },
    /// Remove one or more installed apps by id.
    Remove {
        /// App ids to remove, e.g. `bellwether remove steam`
        ids: Vec<String>,
        /// Remove every currently-installed app in the catalog.
        #[arg(long)]
        all: bool,
        /// Remove a named profile's apps instead (only ones actually installed).
        #[arg(long)]
        profile: Option<String>,
    },
    /// Reinstall and re-apply config for apps that are misbehaving,
    /// overwriting any config drift back to bellwether's defaults.
    Repair {
        /// App ids to repair, e.g. `bellwether repair btop`
        ids: Vec<String>,
        /// Repair every currently-installed app in the catalog.
        #[arg(long)]
        all: bool,
        /// Repair a named profile's apps instead (only ones actually installed).
        #[arg(long)]
        profile: Option<String>,
    },
    /// Launch the interactive terminal UI (checklist + mouse clicks).
    Tui,
    /// Download a Direct-install app into a temporary holding area
    /// (~/Downloads/bellwether-temp/<id>) instead of installing it
    /// properly. Auto-clears after ~48h (override with
    /// $BELLWETHER_TEMP_TTL_HOURS) unless you `promote` it first.
    Temp {
        /// App ids to temp-install, e.g. `bellwether temp my-tool`
        ids: Vec<String>,
    },
    /// Turn a temp install into a proper one: moves it to ~/.local/bin
    /// and adds a desktop-launcher entry so app search can find it.
    Promote {
        /// App ids to promote, e.g. `bellwether promote my-tool`
        ids: Vec<String>,
        /// This is a graphical app — launch it directly instead of via a terminal.
        #[arg(long)]
        gui: bool,
    },
}
