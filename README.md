# 🐑 Bellwether

A little Rust library + installer for getting a fresh Linux box into the
exact shape you want, in one go: monitoring tools, power management, and
your everyday apps — installed *and* configured, not just installed.

Named after the bellwether: the lead sheep of the flock, the one wearing the
bell that all the others follow. This tool leads your system's config into
line the same way.

## What it does

- **Detects your distro's package manager** (apt, pacman, dnf) and, on
  Arch, whatever AUR helper you have (`yay`/`paru`) — no need to tell it
  what you're running.
- **Picks the best install method per app** — native package, AUR,
  Flatpak, or a direct download — in that order of preference, falling
  back automatically if one isn't available.
- **Applies sane post-install config**, not just `pkg install foo` and a
  shrug: btop gets a readable theme and useful panels out of the box,
  PowerTOP auto-tunes on every boot, the CPU governor gets set and kept
  set, Bambu Studio gets a udev rule so you don't need `sudo` for your
  printer's USB connection.
- Ships as a **CLI**, a **TUI** (checklist with mouse-click support), and
  a **one-line install script**.

## Catalog (so far)

| id | what it is | category |
|---|---|---|
| `btop` | resource monitor | Monitoring |
| `powertop` | power usage diagnostics + auto-tune | Power |
| `cpupower` | CPU frequency governor | Power |
| `auto-cpufreq` | automatic CPU speed/power optimizer | Power |
| `zram` | compressed RAM swap instead of disk swap | Power |
| `fish` | friendly interactive shell | Shell & Terminal |
| `starship` | fast, minimal shell prompt | Shell & Terminal |
| `tmux` | terminal multiplexer | Shell & Terminal |
| `neovim` | modernized Vim | Shell & Terminal |
| `ripgrep` | fast recursive grep | Shell & Terminal |
| `fzf` | command-line fuzzy finder | Shell & Terminal |
| `bat` | cat with syntax highlighting | Shell & Terminal |
| `fastfetch` | terminal system-info splash | Shell & Terminal |
| `docker` | container runtime | Server |
| `docker-compose` | multi-container orchestration | Server |
| `ufw` | firewall front-end | Server |
| `fail2ban` | bans IPs after repeated bad logins | Server |
| `purge-snap` | removes snapd and pins apt against it | System Utilities |
| `purge-apport` | removes Ubuntu's crash-reporting daemon | The Pile |
| `purge-whoopsie` | removes Ubuntu's telemetry uploader | The Pile |
| `trim-journal` | trims the systemd journal to 7 days | The Pile |
| `clear-thumbnail-cache` | wipes the file-manager thumbnail cache | The Pile |
| `empty-trash` | empties the desktop trash bin | The Pile |
| `docker-prune` | prunes dangling Docker images & volumes | The Pile |
| `bambustudio` | 3D-printer slicer | Creative |
| `steam` | game store/launcher | Gaming |
| `zen-browser` | privacy-focused Firefox fork | Browsers |

**The Pile** is a handful of debloat/cleanup actions ported over from a
sister project ([Bullshit](https://github.com/sheepy-user1/bs)) — each one
runs as a `Script` action rather than installing anything. They're
reachable from the **All** pen in the TUI; they're not in any profile by
default since they're maintenance actions, not everyday installs.

Adding a new app is a matter of adding one more entry to
`crates/bellwether-core/src/catalog/community.rs` (or `my_apps.rs` for your
own stuff) — see those files, they're plain Rust lists with comments.

## Profiles

Instead of listing app ids by hand every time, install a curated bundle:

```bash
bellwether profiles                      # see what's available
bellwether install --profile home         # everyday desktop use
bellwether install --profile advanced    # home + a fuller terminal toolkit
bellwether install --profile server      # docker, ufw, fail2ban, monitoring
```

`--profile` also works on `remove` and `repair` (only touches apps that
are actually installed). In the TUI, press `1`/`2`/`3` to load a profile's
apps into your current selection, then `i` to install the lot.

## Install

**Script (recommended):**

```bash
curl -fsSL https://raw.githubusercontent.com/sheepy-user1/bellwether/main/scripts/install.sh | bash
```

**From source:**

```bash
git clone https://github.com/sheepy-user1/bellwether.git
cd bellwether
cargo build --release -p bellwether-cli
./target/release/bellwether tui
```

**Prebuilt binary:** grab one from the [Releases](../../releases) page.

## Usage

```bash
bellwether list               # see everything in the catalog
bellwether doctor              # check what this machine can install, and how
bellwether scan                # check what's actually installed right now
bellwether install btop steam  # install specific apps
bellwether install --all       # install everything
bellwether repair btop         # reinstall + force-reapply config (fixes drift/breakage)
bellwether remove steam        # uninstall specific apps
bellwether tui                 # Drover's Yard — the interactive checklist
```

### Drover's Yard (the TUI)

`bellwether tui` opens **Drover's Yard** — a barnyard-themed checklist
(a drover is the person who drives livestock to market, following the
bellwether), organized into four pens:

- **Home** — everyday desktop use (monitoring, power tuning, Bambu Studio, browser, games, fish/starship)
- **Advanced** — Home, plus a fuller terminal toolkit
- **Server** — headless-box essentials (Docker, ufw, fail2ban, terminal tools)
- **All** — the entire catalog, including one-off system utilities like the snap-purge action and anything in `my_apps.rs`

It takes a headcount of your system on startup so every app is tagged
`IN THE BARN` (installed) or `OUT TO PASTURE` (not), then:

- `space` / mouse click — pick apps
- `a` — round up the herd (select/deselect all in the current pen)
- `Tab` or `1`/`2`/`3`/`4` — switch pens (Home / Advanced / Server / All)
- `i` — bring in (install) what's picked
- `r` — call the vet (repair/reinstall) what's picked
- `x` — send to pasture (remove) what's picked — asks for a second `x` to confirm
- `q` — leave the yard

## Project layout

```
crates/
  bellwether-core/   # catalog, package-manager detection, install + config engine (no I/O policy, pure logic + process calls)
  bellwether-cli/    # clap-based CLI + ratatui TUI, the actual binary
scripts/
  install.sh         # curl | bash installer
.github/workflows/
  ci.yml             # build + fmt + clippy on every push
  release.yml        # builds x86_64/aarch64 binaries and attaches to GitHub Releases on a `vX.Y.Z` tag
```

## Adding your own apps (hosted on your own GitHub Releases)

Open `crates/bellwether-core/src/catalog/my_apps.rs` — that file has a
ready-to-copy template and is kept separate from the built-in catalog so
your stuff never gets tangled up with community app updates.

The short version: point `install.direct.url` at
`https://github.com/<you>/<repo>/releases/latest/download/<asset-name>`
(not a URL with a version number baked in). GitHub always redirects that
to whatever you most recently tagged, as long as every release attaches
an asset with that exact filename. Set up your app's own release workflow
the same way `.github/workflows/release.yml` does it here, and it just
works — no need to touch bellwether again when you ship v2.

## Adding a community app to the catalog

Each app is one `AppDef` in `catalog.rs`:

```rust
AppDef {
    id: "my-app",
    name: "My App",
    category: Category::Monitoring,
    description: "what it does",
    install: InstallSpec {
        apt: Some("my-app"),
        pacman: Some("my-app"),
        dnf: Some("my-app"),
        aur: None,
        flatpak: Some("org.example.MyApp"),
        direct: None,
        preference: &[InstallMethod::Native, InstallMethod::Flatpak],
    },
    post_install: &[
        PostInstallStep::WriteUserFile { rel_path: ".config/my-app/config", content: "...", force: false },
        PostInstallStep::Note("anything the user should know"),
    ],
},
```

## License

MIT — see [LICENSE](LICENSE).
