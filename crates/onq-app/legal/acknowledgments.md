# Credits and Attribution

## Copyright

onQ is © VisorCraft LLC and contributors, licensed under the
[GNU General Public License v3.0](LICENSE) (GPL-3.0-only).

## Product

onQ is a multi-platform **search-oriented encrypted prompt vault**.
It keeps prompts as Markdown with YAML frontmatter, builds a local
encrypted MongrelDB index for hybrid keyword + vector search, and
opens from a global shortcut or system tray.

Source repository: https://github.com/visorcraft/onQ

## Runtime dependencies

On Linux desktops the UI shell links against system libraries provided by
the host (WebKitGTK, GTK, GLib, and related stacks). Downstream packagers
(AppImage, distro packages) handle redistribution of those shared objects.
Full license texts for major runtime components are available under the
Licenses page **Runtime components** tab and summarized in Credits.

| Component | Typical license | Project |
| --- | --- | --- |
| WebKitGTK / WRY | LGPL-2.1+ / BSD (mixed) | https://webkitgtk.org |
| GTK 3 | LGPL-2.1-or-later | https://gtk.org |
| GLib / GObject | LGPL-2.1-or-later | https://gtk.org |
| Cairo | LGPL-2.1 / MPL-1.1 | https://cairographics.org |
| libsoup | LGPL-2.1-or-later | https://libsoup.org |
| OpenSSL (system) | Apache-2.0 | https://openssl.org |
| Microsoft Edge WebView2 | Microsoft proprietary | https://developer.microsoft.com/microsoft-edge/webview2/ |
| WebKit / WKWebView | Apple system terms | https://webkit.org |

## Direct product dependencies

onQ is built with Rust, Tauri 2, Svelte 5, and MongrelDB for local storage
and hybrid search. See Credits for complete crate and npm inventories.
