//! Your own apps go here.
//!
//! Each app you build gets one `AppDef` below, pointed at a GitHub
//! Releases asset in ITS OWN repo. The trick: use the `.../releases/
//! latest/download/<asset-name>` URL, NOT a URL with a version number in
//! it. GitHub always redirects that URL to whatever your most recent
//! release actually is — so once you set this up, you never have to touch
//! bellwether again just because you tagged a new version. The only
//! requirement is that every release you publish attaches an asset with
//! that exact same filename (e.g. always call it `my-tool-linux-x86_64`,
//! whatever version it is).
//!
//! If you use the same GitHub Actions pattern bellwether itself uses
//! (see `.github/workflows/release.yml` in this repo) — build on tag
//! push, name the output asset the same thing every time — this just
//! works out of the box.
//!
//! Copy the template below, fill in your details, and add the app to
//! `MY_APPS`.

use crate::model::*;

/*
AppDef {
    id: "my-tool",                       // short, unique, lowercase-with-dashes
    name: "My Tool",                     // display name
    category: Category::Custom,
    description: "One line describing what it does.",
    install: InstallSpec {
        apt: None,
        pacman: None,
        dnf: None,
        aur: None,
        flatpak: None,
        direct: Some(DirectInstall {
            url: "https://github.com/sheepy-user1/my-tool/releases/latest/download/my-tool-linux-x86_64",
            kind: DirectKind::AppImage,     // or DirectKind::Deb / DirectKind::TarGz
            install_name: "my-tool",        // final filename under ~/.local/bin
        }),
        script: None,
        preference: &[InstallMethod::Direct],
    },
    post_install: &[
        PostInstallStep::Note("installed to ~/.local/bin — make sure that's on your PATH"),
    ],
    bin_name: None,   // set Some("actual-binary-name") if it differs from `id`
},
*/

pub const MY_APPS: &[AppDef] = &[
    // Add your own AppDef entries here, using the template above as a guide.
];
