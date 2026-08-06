use crate::model::*;

/// The built-in, "everyone probably wants this" catalog. This is a plain
/// Rust list rather than an external TOML file for now — trivial to
/// extend, and the compiler catches typos in field names.
///
/// If you're adding your *own* apps (things you built yourself), put them
/// in `my_apps.rs` instead — keeps this file as the "community" set that's
/// easy to diff/update independently of your personal stuff.
pub const COMMUNITY_APPS: &[AppDef] = &[
    // ---------------------------------------------------------------
    // Monitoring
    // ---------------------------------------------------------------
    AppDef {
        id: "btop",
        name: "btop++",
        category: Category::Monitoring,
        description: "Resource monitor: CPU, memory, disks, network, and process viewer with a slick TUI.",
        install: InstallSpec {
            apt: Some("btop"),
            pacman: Some("btop"),
            dnf: Some("btop"),
            aur: None,
            flatpak: Some("io.github.aristocratos.btop"),
            direct: None,
            script: None,
            preference: &[InstallMethod::Native, InstallMethod::Flatpak],
        },
        post_install: &[
            PostInstallStep::WriteUserFile {
                rel_path: ".config/btop/btop.conf",
                content: BTOP_CONF,
                force: false,
            },
            PostInstallStep::Note("btop config written with per-core CPU graphs and a readable theme"),
        ],
        bin_name: None,
    },
    // ---------------------------------------------------------------
    // Power management
    // ---------------------------------------------------------------
    AppDef {
        id: "powertop",
        name: "PowerTOP",
        category: Category::Power,
        description: "Diagnoses power usage and can auto-tune devices into power-saving states.",
        install: InstallSpec {
            apt: Some("powertop"),
            pacman: Some("powertop"),
            dnf: Some("powertop"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::RootShell(POWERTOP_UNIT_INSTALL),
            PostInstallStep::EnableService { unit: "powertop-autotune.service", start_now: true },
            PostInstallStep::Note("powertop --auto-tune now runs once at every boot"),
        ],
        bin_name: None,
    },
    AppDef {
        id: "cpupower",
        name: "CPU frequency governor (cpupower)",
        category: Category::Power,
        description: "Sets and persists a sane CPU scaling governor (schedutil where available).",
        install: InstallSpec {
            apt: Some("linux-tools-generic"),
            pacman: Some("cpupower"),
            dnf: Some("kernel-tools"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::RootShell(CPUPOWER_UNIT_INSTALL),
            PostInstallStep::EnableService { unit: "bellwether-cpugovernor.service", start_now: true },
            PostInstallStep::Note("governor set to 'schedutil' (falls back to 'ondemand' if unsupported), reapplied on every boot"),
        ],
        bin_name: None,
    },
    AppDef {
        id: "auto-cpufreq",
        name: "auto-cpufreq",
        category: Category::Power,
        description: "Automatic CPU speed & power optimizer based on usage and battery state.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: Some("auto-cpufreq"),
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Aur],
        },
        post_install: &[
            PostInstallStep::Note(
                "installed via AUR only for now; finish setup by running: sudo auto-cpufreq --install",
            ),
        ],
        bin_name: None,
    },
    // ---------------------------------------------------------------
    // Creative / 3D printing
    // ---------------------------------------------------------------
    AppDef {
        id: "bambustudio",
        name: "Bambu Studio",
        category: Category::Creative,
        description: "Slicer for Bambu Lab (and other) 3D printers.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: Some("bambu-studio-bin"),
            flatpak: Some("com.bambulab.BambuStudio"),
            direct: None,
            script: None,
            preference: &[InstallMethod::Flatpak, InstallMethod::Aur],
        },
        post_install: &[
            PostInstallStep::RootShell(BAMBU_UDEV_RULE_INSTALL),
            PostInstallStep::Note("added a udev rule so your user account can access USB-connected printers without sudo"),
        ],
        bin_name: Some("bambu-studio"),
    },
    // ---------------------------------------------------------------
    // Gaming
    // ---------------------------------------------------------------
    AppDef {
        id: "steam",
        name: "Steam",
        category: Category::Gaming,
        description: "Valve's game store and launcher, with Proton for Windows-game compatibility.",
        install: InstallSpec {
            apt: Some("steam"),
            pacman: Some("steam"),
            dnf: Some("steam"),
            aur: None,
            flatpak: Some("com.valvesoftware.Steam"),
            direct: None,
            script: None,
            preference: &[InstallMethod::Native, InstallMethod::Flatpak],
        },
        post_install: &[
            PostInstallStep::Note("enable 'Steam Play for all titles' in Settings > Compatibility for the widest Proton coverage"),
        ],
        bin_name: None,
    },
    // ---------------------------------------------------------------
    // Browsers
    // ---------------------------------------------------------------
    AppDef {
        id: "zen-browser",
        name: "Zen Browser",
        category: Category::Browser,
        description: "Firefox-based browser focused on speed, privacy, and a calmer UI.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: Some("zen-browser-bin"),
            flatpak: Some("app.zen_browser.zen"),
            direct: None,
            script: None,
            preference: &[InstallMethod::Flatpak, InstallMethod::Aur],
        },
        post_install: &[
            PostInstallStep::Note("set as default browser from within Zen's own settings, or run: xdg-settings set default-web-browser app.zen_browser.zen.desktop"),
        ],
        bin_name: Some("zen"),
    },
    // ---------------------------------------------------------------
    // Shell & Terminal
    // ---------------------------------------------------------------
    AppDef {
        id: "fish",
        name: "fish shell",
        category: Category::Shell,
        description: "Friendly interactive shell: sane defaults, autosuggestions, no config required.",
        install: InstallSpec {
            apt: Some("fish"),
            pacman: Some("fish"),
            dnf: Some("fish"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::Note("to make it your login shell, run: chsh -s $(which fish)  — then log out and back in"),
        ],
        bin_name: None,
    },
    AppDef {
        id: "starship",
        name: "Starship prompt",
        category: Category::Shell,
        description: "Fast, minimal, infinitely customizable shell prompt. Works with bash, zsh, fish.",
        install: InstallSpec {
            apt: None,
            pacman: Some("starship"),
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: Some(STARSHIP_INSTALL_SCRIPT),
            preference: &[InstallMethod::Native, InstallMethod::Script],
        },
        post_install: &[
            PostInstallStep::Note("add this to your shell's rc file to turn it on — bash/zsh: eval \"$(starship init bash)\" (or zsh); fish: starship init fish | source"),
        ],
        bin_name: None,
    },
    AppDef {
        id: "tmux",
        name: "tmux",
        category: Category::Shell,
        description: "Terminal multiplexer: split panes, detachable sessions that survive a disconnect.",
        install: InstallSpec {
            apt: Some("tmux"),
            pacman: Some("tmux"),
            dnf: Some("tmux"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[],
        bin_name: None,
    },
    AppDef {
        id: "neovim",
        name: "Neovim",
        category: Category::Shell,
        description: "Modernized, extensible Vim — the terminal text editor.",
        install: InstallSpec {
            apt: Some("neovim"),
            pacman: Some("neovim"),
            dnf: Some("neovim"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[],
        bin_name: Some("nvim"),
    },
    AppDef {
        id: "ripgrep",
        name: "ripgrep",
        category: Category::Shell,
        description: "Recursive grep replacement — respects .gitignore and is dramatically faster.",
        install: InstallSpec {
            apt: Some("ripgrep"),
            pacman: Some("ripgrep"),
            dnf: Some("ripgrep"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[],
        bin_name: Some("rg"),
    },
    AppDef {
        id: "fzf",
        name: "fzf",
        category: Category::Shell,
        description: "Command-line fuzzy finder — pipe anything into it, filter interactively.",
        install: InstallSpec {
            apt: Some("fzf"),
            pacman: Some("fzf"),
            dnf: Some("fzf"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::Note("add key bindings to your shell rc — see fzf --bash / --zsh / --fish"),
        ],
        bin_name: None,
    },
    AppDef {
        id: "bat",
        name: "bat",
        category: Category::Shell,
        description: "cat replacement with syntax highlighting and git-diff markers in the gutter.",
        install: InstallSpec {
            apt: Some("bat"),
            pacman: Some("bat"),
            dnf: Some("bat"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::Note("on Debian/Ubuntu the binary is installed as 'batcat' (name clash with an old package) — alias bat=batcat if you want the short name"),
        ],
        bin_name: None,
    },
    AppDef {
        id: "fastfetch",
        name: "fastfetch",
        category: Category::Shell,
        description: "Fast system-info splash for your terminal — the neofetch successor.",
        install: InstallSpec {
            apt: Some("fastfetch"),
            pacman: Some("fastfetch"),
            dnf: Some("fastfetch"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::Note("older LTS releases may not carry this package yet — check your distro's version if apt/dnf can't find it"),
        ],
        bin_name: None,
    },
    // ---------------------------------------------------------------
    // Server
    // ---------------------------------------------------------------
    AppDef {
        id: "docker",
        name: "Docker Engine",
        category: Category::Server,
        description: "Container runtime: build and run containers.",
        install: InstallSpec {
            apt: Some("docker.io"),
            pacman: Some("docker"),
            dnf: Some("docker"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::EnableService { unit: "docker.service", start_now: true },
            PostInstallStep::RootShell(DOCKER_GROUP_ADD),
            PostInstallStep::Note("log out and back in (or run 'newgrp docker') for the docker group membership to take effect — until then, docker commands need sudo"),
        ],
        bin_name: None,
    },
    AppDef {
        id: "docker-compose",
        name: "Docker Compose",
        category: Category::Server,
        description: "Define and run multi-container Docker applications from a single YAML file.",
        install: InstallSpec {
            apt: Some("docker-compose-plugin"),
            pacman: Some("docker-compose"),
            dnf: Some("docker-compose"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::Note("modern installs expose this as 'docker compose' (no hyphen); the old 'docker-compose' binary may not exist depending on your distro's package"),
        ],
        bin_name: Some("docker"),
    },
    AppDef {
        id: "ufw",
        name: "UFW (Uncomplicated Firewall)",
        category: Category::Server,
        description: "Simple firewall front-end for iptables/nftables — deny incoming, allow outgoing, by default.",
        install: InstallSpec {
            apt: Some("ufw"),
            pacman: Some("ufw"),
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::RootShell(UFW_SANE_DEFAULTS),
            PostInstallStep::Note("default policy set to deny incoming / allow outgoing, then enabled. Fedora/dnf systems use firewalld instead — not available here."),
        ],
        bin_name: None,
    },
    AppDef {
        id: "fail2ban",
        name: "Fail2ban",
        category: Category::Server,
        description: "Bans IPs that show malicious signs, like too many failed login attempts.",
        install: InstallSpec {
            apt: Some("fail2ban"),
            pacman: Some("fail2ban"),
            dnf: Some("fail2ban"),
            aur: None,
            flatpak: None,
            direct: None,
            script: None,
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::EnableService { unit: "fail2ban.service", start_now: true },
            PostInstallStep::Note("running with fail2ban's defaults — add a jail.local for SSH-specific tuning"),
        ],
        bin_name: Some("fail2ban-client"),
    },
    // ---------------------------------------------------------------
    // System Utilities
    // ---------------------------------------------------------------
    AppDef {
        id: "purge-snap",
        name: "Remove Snap (snapd)",
        category: Category::Utilities,
        description: "Purges snapd and any snap packages, and pins apt so nothing quietly reinstalls it (Debian/Ubuntu only).",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: Some(PURGE_SNAP_SCRIPT),
            preference: &[InstallMethod::Script],
        },
        post_install: &[
            PostInstallStep::Note("snapd removed; apt is pinned to refuse reinstalling it as a dependency of anything else"),
        ],
        // Reused as a presence check: bin_name "snap" means the IN THE
        // BARN / OUT TO PASTURE tag actually reports whether snap is
        // currently present, rather than whether this "app" (an action,
        // not a package) has been run before.
        bin_name: Some("snap"),
    },
    // ---------------------------------------------------------------
    // The Pile — debloat actions ported over from the Bullshit project.
    // Each of these is a Script action, same pattern as purge-snap above:
    // no package to fetch, just something to shovel out. Grade noted in
    // the description (A = safe as houses, B = know what you're losing).
    // ---------------------------------------------------------------
    AppDef {
        id: "purge-apport",
        name: "Apport (crash reporting)",
        category: Category::Debloat,
        description: "[Grade A] Catches app crashes and offers to send reports to Ubuntu's servers. Most people never see the popup — it just burns disk in /var/crash and a bit of CPU on every crash. Ubuntu/Debian only.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: Some(PURGE_APPORT_SCRIPT),
            preference: &[InstallMethod::Script],
        },
        post_install: &[PostInstallStep::Note("apport disabled, removed, and /var/crash cleared")],
        bin_name: Some("apport-bug"),
    },
    AppDef {
        id: "purge-whoopsie",
        name: "Whoopsie & ubuntu-report",
        category: Category::Debloat,
        description: "[Grade A] Whoopsie uploads Apport's crash reports to Canonical; ubuntu-report sends anonymised install stats home at first boot. Neither does anything for you day to day. Ubuntu only.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: Some(PURGE_WHOOPSIE_SCRIPT),
            preference: &[InstallMethod::Script],
        },
        post_install: &[],
        bin_name: Some("whoopsie"),
    },
    AppDef {
        id: "trim-journal",
        name: "Trim the systemd journal",
        category: Category::Debloat,
        description: "[Grade A] journald logs kernel and service messages to /var/log/journal and can quietly grow to gigabytes on a long-running box. Trims it down to the last 7 days.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: Some("journalctl --vacuum-time=7d"),
            preference: &[InstallMethod::Script],
        },
        post_install: &[],
        bin_name: None,
    },
    AppDef {
        id: "clear-thumbnail-cache",
        name: "Clear thumbnail cache",
        category: Category::Debloat,
        description: "[Grade A] Your file manager caches a thumbnail for every image and video you've ever browsed in ~/.cache/thumbnails. It regenerates automatically — free space, zero real downside.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: Some(CLEAR_THUMBNAIL_CACHE_SCRIPT),
            preference: &[InstallMethod::Script],
        },
        post_install: &[],
        bin_name: None,
    },
    AppDef {
        id: "empty-trash",
        name: "Empty the trash bin",
        category: Category::Debloat,
        description: "[Grade A] Files 'deleted' from a graphical file manager usually just move to ~/.local/share/Trash and sit there indefinitely instead of actually being freed.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: Some(EMPTY_TRASH_SCRIPT),
            preference: &[InstallMethod::Script],
        },
        post_install: &[],
        bin_name: None,
    },
    AppDef {
        id: "docker-prune",
        name: "Prune dangling Docker images & volumes",
        category: Category::Debloat,
        description: "[Grade B] Every rebuild leaves old image layers behind as 'dangling', and stopped containers keep volumes around until cleaned up. On a dev box this quietly eats tens of gigabytes. Know what you're losing — this removes anything not currently referenced by a running container.",
        install: InstallSpec {
            apt: None,
            pacman: None,
            dnf: None,
            aur: None,
            flatpak: None,
            direct: None,
            script: Some("command -v docker >/dev/null 2>&1 && docker system prune -af --volumes || echo \"docker not found — nothing to prune\""),
            preference: &[InstallMethod::Script],
        },
        post_install: &[],
        bin_name: Some("docker"),
    },
];

// ---------------------------------------------------------------------
// Config file / shell script bodies, kept at the bottom to keep the
// catalog table itself scannable.
// ---------------------------------------------------------------------

const BTOP_CONF: &str = r#"# Written by bellwether — a "just works" btop config
color_theme = "gruvbox_dark"
theme_background = true
truecolor = true
vim_keys = false
graph_symbol = "braille"
shown_boxes = "cpu mem net proc"
update_ms = 1500
proc_sorting = "cpu lazy"
proc_tree = true
proc_colors = true
proc_gradient = true
cpu_graph_upper = "total"
cpu_graph_lower = "total"
cpu_single_graph = false
show_cpu_freq = true
clock_format = "%H:%M"
background_update = true
show_battery = true
show_disks = true
disk_free_priv = false
net_download = 100
net_upload = 100
net_auto = true
"#;

const POWERTOP_UNIT_INSTALL: &str = r#"cat > /etc/systemd/system/powertop-autotune.service << 'EOF'
[Unit]
Description=PowerTOP auto tune
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/usr/sbin/powertop --auto-tune

[Install]
WantedBy=multi-user.target
EOF
systemctl daemon-reload"#;

const CPUPOWER_UNIT_INSTALL: &str = r#"cat > /etc/systemd/system/bellwether-cpugovernor.service << 'EOF'
[Unit]
Description=Set CPU scaling governor (bellwether)
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/bin/sh -c 'cpupower frequency-set -g schedutil || cpupower frequency-set -g ondemand || true'

[Install]
WantedBy=multi-user.target
EOF
systemctl daemon-reload"#;

const BAMBU_UDEV_RULE_INSTALL: &str = r#"cat > /etc/udev/rules.d/99-bambu-printer.rules << 'EOF'
# Allow non-root USB access to Bambu Lab printers (installed by bellwether)
SUBSYSTEM=="usb", ATTRS{idVendor}=="28e9", MODE="0666", GROUP="plugdev"
EOF
udevadm control --reload-rules
udevadm trigger"#;

const DOCKER_GROUP_ADD: &str =
    r#"if [ -n "$SUDO_USER" ]; then usermod -aG docker "$SUDO_USER"; fi"#;

const UFW_SANE_DEFAULTS: &str = r#"ufw default deny incoming
ufw default allow outgoing
ufw --force enable"#;

// Purges snapd on Debian/Ubuntu-family systems and pins apt so a later
// `apt install` of something that recommends snapd doesn't quietly bring
// it back. Deliberately conservative: does nothing if snap isn't present,
// and does nothing on non-apt systems.
const PURGE_SNAP_SCRIPT: &str = r#"if command -v snap >/dev/null 2>&1 && command -v apt-get >/dev/null 2>&1; then
  systemctl stop snapd.service snapd.socket 2>/dev/null || true
  for pkg in $(snap list 2>/dev/null | awk 'NR>1{print $1}'); do
    snap remove --purge "$pkg" 2>/dev/null || true
  done
  apt-get purge -y snapd
  rm -rf /var/cache/snapd /snap /var/snap /var/lib/snapd
  cat > /etc/apt/preferences.d/nosnap.pref << 'EOF'
Package: snapd
Pin: release a=*
Pin-Priority: -10
EOF
else
  echo "snap not found (or not an apt system) — nothing to purge"
fi"#;

// Official Starship installer, run non-interactively. Used only as a
// fallback when there's no native package (e.g. apt/dnf don't carry it).
const STARSHIP_INSTALL_SCRIPT: &str = r#"curl -sS https://starship.rs/install.sh | sh -s -- -y"#;

const PURGE_APPORT_SCRIPT: &str = r#"if command -v apt-get >/dev/null 2>&1; then
  systemctl disable --now apport.service 2>/dev/null || true
  apt-get purge -y apport apport-symptoms
  rm -rf /var/crash
else
  echo "apport is Ubuntu/Debian-specific — nothing to do here"
fi"#;

const PURGE_WHOOPSIE_SCRIPT: &str = r#"if command -v apt-get >/dev/null 2>&1; then
  apt-get purge -y whoopsie ubuntu-report
else
  echo "whoopsie is Ubuntu-specific — nothing to do here"
fi"#;

// These two touch the *invoking user's* home directory, not root's — same
// $SUDO_USER trick used for the docker group setup above, since bellwether
// always runs Script actions with root privileges.
const CLEAR_THUMBNAIL_CACHE_SCRIPT: &str = r#"target_home="${SUDO_USER:+/home/$SUDO_USER}"
target_home="${target_home:-$HOME}"
rm -rf "$target_home/.cache/thumbnails/"*"#;

const EMPTY_TRASH_SCRIPT: &str = r#"target_home="${SUDO_USER:+/home/$SUDO_USER}"
target_home="${target_home:-$HOME}"
rm -rf "$target_home/.local/share/Trash/"*"#;
