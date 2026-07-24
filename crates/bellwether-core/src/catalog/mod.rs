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
