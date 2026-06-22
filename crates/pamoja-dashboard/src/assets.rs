//! The static page assets, served either embedded or live from disk.
//!
//! In production the page is baked into the firmware with `include_bytes!`, so there
//! is no filesystem dependency and the dashboard is part of the image. In development
//! the same directory is read from disk on every request, so editing the app and
//! reloading shows the change with no recompile. The dashboard is a multi-file
//! ES-module zQuery app, so both modes resolve any nested path under the web root.

#[cfg(feature = "serve")]
use std::path::PathBuf;

/// One bundled file: its URL path, its MIME type, and its bytes.
struct Asset {
    path: &'static str,
    content_type: &'static str,
    bytes: &'static [u8],
}

const HTML: &str = "text/html; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "application/javascript; charset=utf-8";

// The bundle, embedded at compile time. `/` maps to the page shell. The order does not
// matter; lookups are by exact path.
const EMBEDDED: &[Asset] = &[
    Asset {
        path: "/",
        content_type: HTML,
        bytes: include_bytes!("../web/index.html"),
    },
    Asset {
        path: "/global.css",
        content_type: CSS,
        bytes: include_bytes!("../web/global.css"),
    },
    Asset {
        path: "/zquery.min.js",
        content_type: JS,
        bytes: include_bytes!("../web/zquery.min.js"),
    },
    Asset {
        path: "/app/app.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/app.js"),
    },
    Asset {
        path: "/app/store.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/store.js"),
    },
    Asset {
        path: "/app/routes.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/routes.js"),
    },
    Asset {
        path: "/app/feed.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/feed.js"),
    },
    Asset {
        path: "/app/edits.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/edits.js"),
    },
    Asset {
        path: "/app/nav.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/nav.js"),
    },
    Asset {
        path: "/app/detail.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/detail.js"),
    },
    Asset {
        path: "/app/i18n.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/i18n.js"),
    },
    Asset {
        path: "/app/viz.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/viz.js"),
    },
    Asset {
        path: "/app/components/top-bar.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/components/top-bar.js"),
    },
    Asset {
        path: "/app/components/dashboard-page.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/components/dashboard-page.js"),
    },
    Asset {
        path: "/app/components/sensor-modal.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/components/sensor-modal.js"),
    },
    Asset {
        path: "/app/components/manage-modal.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/components/manage-modal.js"),
    },
    Asset {
        path: "/app/components/group-modal.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/components/group-modal.js"),
    },
    Asset {
        path: "/app/components/mesh-modal.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/components/mesh-modal.js"),
    },
    Asset {
        path: "/app/components/network-view.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/components/network-view.js"),
    },
    Asset {
        path: "/app/components/alarm-bar.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/components/alarm-bar.js"),
    },
    Asset {
        path: "/app/i18n/en.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/i18n/en.js"),
    },
    Asset {
        path: "/app/i18n/sw.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/i18n/sw.js"),
    },
    Asset {
        path: "/app/i18n/ar.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/i18n/ar.js"),
    },
    Asset {
        path: "/app/i18n/fr.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/i18n/fr.js"),
    },
    Asset {
        path: "/app/i18n/pt.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/i18n/pt.js"),
    },
    Asset {
        path: "/app/i18n/hi.js",
        content_type: JS,
        bytes: include_bytes!("../web/app/i18n/hi.js"),
    },
];

// The MIME type for a file, by extension, for the directory (development) mode.
fn mime_for(path: &str) -> &'static str {
    if path.ends_with(".html") {
        HTML
    } else if path.ends_with(".css") {
        CSS
    } else if path.ends_with(".js") || path.ends_with(".mjs") {
        JS
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}

/// Where the page assets come from.
#[derive(Clone, Debug)]
pub enum Assets {
    /// Baked into the binary at compile time: the production path.
    Embedded,
    /// Read from a directory on each request: the hot-reloading development path.
    #[cfg(feature = "serve")]
    Dir(PathBuf),
}

impl Assets {
    /// Resolves a request path to a file's MIME type and bytes.
    ///
    /// The request path `"/"` resolves to the page shell. In [`Assets::Dir`] mode any
    /// file under the directory is served (typed by extension), read fresh from disk so
    /// edits show up on reload; a path that escapes the directory resolves to `None`.
    ///
    /// # Arguments
    ///
    /// * `path` - the request path, such as `"/app/app.js"`.
    ///
    /// # Returns
    ///
    /// The MIME type and the file's bytes, or `None` if no asset matches.
    pub fn get(&self, path: &str) -> Option<(&'static str, Vec<u8>)> {
        match self {
            Assets::Embedded => EMBEDDED
                .iter()
                .find(|a| a.path == path)
                .map(|a| (a.content_type, a.bytes.to_vec())),
            #[cfg(feature = "serve")]
            Assets::Dir(root) => {
                let relative = if path == "/" {
                    "index.html"
                } else {
                    path.trim_start_matches('/')
                };
                // Refuse anything that tries to climb out of the asset directory.
                if relative.contains("..") {
                    return None;
                }
                let bytes = std::fs::read(root.join(relative)).ok()?;
                Some((mime_for(relative), bytes))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_serves_the_shell_for_root() {
        let (content_type, bytes) = Assets::Embedded.get("/").expect("shell present");
        assert_eq!(content_type, HTML);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn embedded_serves_the_app_entry_and_framework() {
        assert!(Assets::Embedded.get("/zquery.min.js").is_some());
        assert!(Assets::Embedded.get("/app/app.js").is_some());
        assert!(Assets::Embedded.get("/global.css").is_some());
    }

    #[test]
    fn embedded_has_all_six_seed_locales() {
        for locale in ["en", "sw", "ar", "fr", "pt", "hi"] {
            let path = format!("/app/i18n/{locale}.js");
            assert!(Assets::Embedded.get(&path).is_some(), "missing {path}");
        }
    }

    #[test]
    fn an_unknown_path_resolves_to_nothing() {
        assert!(Assets::Embedded.get("/secret").is_none());
    }

    #[test]
    fn mime_types_follow_the_extension() {
        assert_eq!(mime_for("index.html"), HTML);
        assert_eq!(mime_for("app/app.js"), JS);
        assert_eq!(mime_for("global.css"), CSS);
    }
}
