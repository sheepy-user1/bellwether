use crate::catalog;
use crate::model::AppDef;

/// A named bundle of catalog app ids, so `bellwether install --profile
/// server` installs a whole curated set in one shot instead of listing
/// every id by hand every time.
pub struct Profile {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub app_ids: &'static [&'static str],
}

pub const PROFILES: &[Profile] = &[
    Profile {
        id: "standard",
        name: "Standard Client",
        description: "Everyday desktop use: a resource monitor, a browser, and your games.",
        app_ids: &["btop", "zen-browser", "steam"],
    },
    Profile {
        id: "advanced",
        name: "Advanced Client",
        description: "Standard, plus power tuning and creative tools for a hands-on desktop.",
        app_ids: &[
            "btop",
            "powertop",
            "cpupower",
            "auto-cpufreq",
            "bambustudio",
            "steam",
            "zen-browser",
        ],
    },
    Profile {
        id: "server",
        name: "Server",
        description:
            "Headless-box essentials: containers, firewall, intrusion prevention, monitoring.",
        app_ids: &["btop", "docker", "docker-compose", "ufw", "fail2ban"],
    },
];

pub fn find_profile(id: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|p| p.id == id)
}

/// Resolves a profile's app ids against the live catalog (community + your
/// own apps). Ids that don't (or no longer) exist in the catalog are
/// silently skipped rather than erroring — keeps profiles resilient to
/// catalog edits.
pub fn profile_apps(profile: &Profile) -> Vec<&'static AppDef> {
    profile
        .app_ids
        .iter()
        .filter_map(|id| catalog::find(id))
        .collect()
}
