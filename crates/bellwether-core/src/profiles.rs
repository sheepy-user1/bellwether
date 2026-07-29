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
        id: "home",
        name: "Home",
        description: "Everyday desktop use: monitoring, power tuning, your 3D printer, browser, games, and a friendlier shell.",
        app_ids: &[
            "btop",
            "fastfetch",
            "powertop",
            "cpupower",
            "auto-cpufreq",
            "bambustudio",
            "steam",
            "zen-browser",
            "fish",
            "starship",
        ],
    },
    Profile {
        id: "advanced",
        name: "Advanced",
        description: "Home, plus a fuller terminal toolkit for hands-on tinkering.",
        app_ids: &[
            "btop",
            "fastfetch",
            "powertop",
            "cpupower",
            "auto-cpufreq",
            "bambustudio",
            "steam",
            "zen-browser",
            "fish",
            "starship",
            "tmux",
            "neovim",
            "ripgrep",
            "fzf",
            "bat",
        ],
    },
    Profile {
        id: "server",
        name: "Server",
        description:
            "Headless-box essentials: containers, firewall, intrusion prevention, monitoring, and a solid terminal toolkit.",
        app_ids: &[
            "btop",
            "docker",
            "docker-compose",
            "ufw",
            "fail2ban",
            "tmux",
            "neovim",
            "ripgrep",
            "fzf",
        ],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_ids_are_unique() {
        let mut ids: Vec<&str> = PROFILES.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        let mut deduped = ids.clone();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len(), "duplicate profile id");
    }

    #[test]
    fn every_profile_app_id_actually_exists_in_the_catalog() {
        // Guards against a typo'd app id in a profile's app_ids list —
        // profile_apps() would otherwise just silently drop it.
        for p in PROFILES {
            for id in p.app_ids {
                assert!(
                    crate::catalog::find(id).is_some(),
                    "profile '{}' references unknown app id '{}'",
                    p.id,
                    id
                );
            }
        }
    }

    #[test]
    fn find_profile_looks_up_by_id() {
        assert!(find_profile("home").is_some());
        assert!(find_profile("advanced").is_some());
        assert!(find_profile("server").is_some());
        assert!(find_profile("does-not-exist").is_none());
    }

    #[test]
    fn advanced_profile_is_a_superset_of_home() {
        let home = find_profile("home").unwrap();
        let advanced = find_profile("advanced").unwrap();
        for id in home.app_ids {
            assert!(
                advanced.app_ids.contains(id),
                "advanced profile is missing '{id}' from home"
            );
        }
    }
}
