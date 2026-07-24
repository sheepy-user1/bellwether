//! bellwether-core: app catalog, package-manager detection, and the
//! install + post-install-configuration engine used by both the
//! bellwether CLI and its TUI.

pub mod catalog;
pub mod configure;
pub mod error;
pub mod installer;
pub mod model;
pub mod sysinfo;

pub use catalog::{catalog, find};
pub use error::{BwError, BwResult};
pub use model::{AppDef, Category, InstallMethod};
pub use sysinfo::SystemInfo;
