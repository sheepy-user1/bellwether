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
| `bambustudio` | 3D-printer slicer | Creative |
| `steam` | game store/launcher | Gaming |
| `zen-browser` | privacy-focused Firefox fork | Browsers |

Adding a new app is a matter of adding one more entry to
`crates/bellwether-core/src/catalog.rs` — see that file, it's a plain Rust
list with comments.

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
bellwether list              # see everything in the catalog
bellwether doctor             # check what this machine can install, and how
bellwether install btop steam # install specific apps
bellwether install --all      # install everything
bellwether tui                # interactive checklist (arrow keys, space, mouse click, i to install)
```

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
