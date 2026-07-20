//! Bundled About / Licenses / Credits content.
//!
//! Long texts and inventory JSON are compile-time embeds under `legal/`.
//! Regenerate inventories with `scripts/regen-credits.sh` after dependency changes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AboutInfo {
    pub app_name: String,
    pub version: String,
    pub license: String,
    pub repository: String,
    pub git_sha: String,
    pub description: String,
    pub tagline: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseDocMeta {
    pub id: String,
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrateCredit {
    pub name: String,
    pub version: String,
    pub license: String,
    #[serde(default)]
    pub repository: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpmPackageCredit {
    pub name: String,
    pub version: String,
    pub license: String,
    #[serde(default)]
    pub repository: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeComponent {
    pub name: String,
    pub notes: String,
    pub licenses: String,
    pub spdx_id: String,
    pub project_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditsData {
    pub crates: Vec<CrateCredit>,
    pub packages: Vec<NpmPackageCredit>,
    pub runtime: Vec<RuntimeComponent>,
    pub crate_count: usize,
    pub package_count: usize,
    pub runtime_count: usize,
}

pub fn about_info() -> AboutInfo {
    AboutInfo {
        app_name: "onQ".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        license: "GPL-3.0-only".into(),
        repository: "https://github.com/visorcraft/onQ".into(),
        git_sha: option_env!("GIT_SHA").unwrap_or("dev").into(),
        description: "Search-oriented encrypted prompt vault.".into(),
        tagline: "Built on Rust + Tauri 2 + Svelte 5.".into(),
        platform: std::env::consts::OS.into(),
    }
}

pub fn license_docs() -> Vec<LicenseDocMeta> {
    vec![
        LicenseDocMeta {
            id: "app".into(),
            title: "onQ license".into(),
            subtitle: "GPL-3.0-only license text bundled into the application.".into(),
        },
        LicenseDocMeta {
            id: "third-party".into(),
            title: "Third-party (Rust)".into(),
            subtitle: "Rust crates included in this build, grouped by license.".into(),
        },
        LicenseDocMeta {
            id: "npm".into(),
            title: "Frontend (npm)".into(),
            subtitle: "JavaScript packages (runtime UI plus build tooling).".into(),
        },
        LicenseDocMeta {
            id: "acknowledgments".into(),
            title: "Acknowledgments".into(),
            subtitle: "Narrative attribution for onQ, runtime components, and direct dependencies."
                .into(),
        },
        LicenseDocMeta {
            id: "runtime".into(),
            title: "Runtime components".into(),
            subtitle: "Full license texts for WebKitGTK, GTK, GLib, and related runtimes.".into(),
        },
    ]
}

pub fn license_document(id: &str) -> Result<String, String> {
    match id {
        "app" | "onq" | "viewer" => Ok(include_str!("../legal/LICENSE-GPL-3.0.txt").into()),
        "third-party" | "rust" => Ok(include_str!("../legal/third-party.md").into()),
        "npm" | "frontend" => Ok(include_str!("../legal/npm-third-party.md").into()),
        "acknowledgments" | "ack" => Ok(include_str!("../legal/acknowledgments.md").into()),
        "runtime" => Ok(include_str!("../legal/runtime.md").into()),
        other => Err(format!("unknown license document: {other}")),
    }
}

fn runtime_components() -> Vec<RuntimeComponent> {
    vec![
        RuntimeComponent {
            name: "WebKitGTK / WRY".into(),
            notes: "Linux WebView backend used by Tauri/WRY".into(),
            licenses: "LGPL-2.1+ / BSD (mixed)".into(),
            spdx_id: "LGPL-2.1-or-later".into(),
            project_url: "https://webkitgtk.org".into(),
        },
        RuntimeComponent {
            name: "GTK 3".into(),
            notes: "Windowing toolkit on Linux".into(),
            licenses: "LGPL-2.1-or-later".into(),
            spdx_id: "LGPL-2.1-or-later".into(),
            project_url: "https://www.gtk.org".into(),
        },
        RuntimeComponent {
            name: "GLib / GObject".into(),
            notes: "Core GObject event loop primitives".into(),
            licenses: "LGPL-2.1-or-later".into(),
            spdx_id: "LGPL-2.1-or-later".into(),
            project_url: "https://www.gtk.org".into(),
        },
        RuntimeComponent {
            name: "Cairo".into(),
            notes: "2D graphics library".into(),
            licenses: "LGPL-2.1 / MPL-1.1".into(),
            spdx_id: "LGPL-2.1-or-later".into(),
            project_url: "https://www.cairographics.org".into(),
        },
        RuntimeComponent {
            name: "libsoup".into(),
            notes: "HTTP client library used with WebKitGTK".into(),
            licenses: "LGPL-2.1-or-later".into(),
            spdx_id: "LGPL-2.1-or-later".into(),
            project_url: "https://libsoup.org".into(),
        },
        RuntimeComponent {
            name: "OpenSSL (system, if present)".into(),
            notes: "TLS when provided by the host stack".into(),
            licenses: "Apache-2.0".into(),
            spdx_id: "Apache-2.0".into(),
            project_url: "https://www.openssl.org".into(),
        },
        RuntimeComponent {
            name: "Microsoft Edge WebView2 (Windows)".into(),
            notes: "Windows WebView host; not redistributed by this package".into(),
            licenses: "Microsoft proprietary (system runtime)".into(),
            spdx_id: "".into(),
            project_url: "https://developer.microsoft.com/microsoft-edge/webview2/".into(),
        },
        RuntimeComponent {
            name: "WebKit / WKWebView (macOS)".into(),
            notes: "macOS WebView host".into(),
            licenses: "Apple system framework terms".into(),
            spdx_id: "".into(),
            project_url: "https://webkit.org".into(),
        },
    ]
}

pub fn credits_data() -> Result<CreditsData, String> {
    let crates: Vec<CrateCredit> =
        serde_json::from_str(include_str!("../legal/crates.json")).map_err(|e| e.to_string())?;
    let packages: Vec<NpmPackageCredit> =
        serde_json::from_str(include_str!("../legal/npm-packages.json"))
            .map_err(|e| e.to_string())?;
    let runtime = runtime_components();
    Ok(CreditsData {
        crate_count: crates.len(),
        package_count: packages.len(),
        runtime_count: runtime.len(),
        crates,
        packages,
        runtime,
    })
}

pub fn runtime_license_text(spdx_id: &str) -> Result<String, String> {
    match spdx_id {
        "LGPL-2.1-or-later" | "LGPL-2.1" => {
            Ok(include_str!("../legal/runtime/LGPL-2.1-or-later.txt").into())
        }
        "LGPL-3.0-only" | "LGPL-3.0" => {
            Ok(include_str!("../legal/runtime/LGPL-3.0-only.txt").into())
        }
        "GPL-2.0-or-later" | "GPL-2.0" => {
            Ok(include_str!("../legal/runtime/GPL-2.0-or-later.txt").into())
        }
        "Apache-2.0" => Ok(include_str!("../legal/runtime/Apache-2.0.txt").into()),
        "" => Err("no bundled text for this system runtime".into()),
        other => Err(format!("unknown runtime SPDX id: {other}")),
    }
}

#[tauri::command]
pub fn about_info_cmd() -> AboutInfo {
    about_info()
}

#[tauri::command]
pub fn license_docs_cmd() -> Vec<LicenseDocMeta> {
    license_docs()
}

#[tauri::command]
pub fn license_document_cmd(id: String) -> Result<String, String> {
    license_document(&id)
}

#[tauri::command]
pub fn credits_data_cmd() -> Result<CreditsData, String> {
    credits_data()
}

#[tauri::command]
pub fn runtime_license_text_cmd(spdx_id: String) -> Result<String, String> {
    runtime_license_text(&spdx_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_info_has_onq_identity() {
        let info = about_info();
        assert_eq!(info.app_name, "onQ");
        assert_eq!(info.license, "GPL-3.0-only");
        assert!(info.repository.contains("onQ"));
    }

    #[test]
    fn license_docs_cover_five_tabs() {
        let docs = license_docs();
        assert_eq!(docs.len(), 5);
        assert!(license_document("app")
            .unwrap()
            .contains("GNU GENERAL PUBLIC LICENSE"));
        assert!(!license_document("third-party").unwrap().is_empty());
        assert!(!license_document("npm").unwrap().is_empty());
    }

    #[test]
    fn credits_data_loads_inventories() {
        let data = credits_data().expect("credits");
        assert!(data.crate_count > 0);
        assert!(data.package_count > 0);
        assert_eq!(data.runtime_count, 8);
    }
}
