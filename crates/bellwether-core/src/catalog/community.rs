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
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::RootShell(POWERTOP_UNIT_INSTALL),
            PostInstallStep::EnableService { unit: "powertop-autotune.service", start_now: true },
            PostInstallStep::Note("powertop --auto-tune now runs once at every boot"),
        ],
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
            preference: &[InstallMethod::Native],
        },
        post_install: &[
            PostInstallStep::RootShell(CPUPOWER_UNIT_INSTALL),
            PostInstallStep::EnableService { unit: "bellwether-cpugovernor.service", start_now: true },
            PostInstallStep::Note("governor set to 'schedutil' (falls back to 'ondemand' if unsupported), reapplied on every boot"),
        ],
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
            preference: &[InstallMethod::Aur],
        },
        post_install: &[
            PostInstallStep::Note(
                "installed via AUR only for now; finish setup by running: sudo auto-cpufreq --install",
            ),
        ],
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
            preference: &[InstallMethod::Flatpak, InstallMethod::Aur],
        },
        post_install: &[
            PostInstallStep::RootShell(BAMBU_UDEV_RULE_INSTALL),
            PostInstallStep::Note("added a udev rule so your user account can access USB-connected printers without sudo"),
        ],
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
            preference: &[InstallMethod::Native, InstallMethod::Flatpak],
        },
        post_install: &[
            PostInstallStep::Note("enable 'Steam Play for all titles' in Settings > Compatibility for the widest Proton coverage"),
        ],
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
            preference: &[InstallMethod::Flatpak, InstallMethod::Aur],
        },
        post_install: &[
            PostInstallStep::Note("set as default browser from within Zen's own settings, or run: xdg-settings set default-web-browser app.zen_browser.zen.desktop"),
        ],
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
