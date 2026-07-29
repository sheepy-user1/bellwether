mod community;
mod my_apps;

pub use community::COMMUNITY_APPS;
pub use my_apps::MY_APPS;

use crate::model::AppDef;

/// The full catalog: built-in community apps plus whatever you've added
/// to `my_apps.rs`. Recomputed on each call rather than cached — it's a
/// handful of pointer copies, not worth the complexity of a static Vec.
pub fn catalog() -> Vec<&'static AppDef> {
    COMMUNITY_APPS.iter().chain(MY_APPS.iter()).collect()
}

pub fn find(id: &str) -> Option<&'static AppDef> {
    catalog().into_iter().find(|a| a.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_duplicate_app_ids() {
        let mut ids: Vec<&str> = catalog().iter().map(|a| a.id).collect();
        ids.sort_unstable();
        let mut deduped = ids.clone();
        deduped.dedup();
        assert_eq!(
            ids.len(),
            deduped.len(),
            "duplicate app id(s) in the catalog — find() would silently \
             shadow one of them"
        );
    }

    #[test]
    fn every_app_defines_at_least_one_install_method() {
        for app in catalog() {
            let has_any = app.install.apt.is_some()
                || app.install.pacman.is_some()
                || app.install.dnf.is_some()
                || app.install.aur.is_some()
                || app.install.flatpak.is_some()
                || app.install.direct.is_some()
                || app.install.script.is_some();
            assert!(
                has_any,
                "app '{}' defines no install method at all — it can never be installed",
                app.id
            );
        }
    }
}
