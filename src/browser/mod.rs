pub mod detector;
pub mod launcher;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Browser {
    Chrome {
        app_path: Option<String>,
    },
    Firefox {
        app_path: Option<String>,
    },
    Brave {
        app_path: Option<String>,
    },
    Edge {
        app_path: Option<String>,
    },
    Safari {
        app_path: Option<String>,
    },
    Arc {
        app_path: Option<String>,
    },
    Orion {
        app_path: Option<String>,
    },
    Other {
        name: String,
        app_path: Option<String>,
    },
}

impl Browser {
    pub fn name(&self) -> &str {
        match self {
            Self::Chrome { .. } => "Chrome",
            Self::Firefox { .. } => "Firefox",
            Self::Brave { .. } => "Brave",
            Self::Edge { .. } => "Edge",
            Self::Safari { .. } => "Safari",
            Self::Arc { .. } => "Arc",
            Self::Orion { .. } => "Orion",
            Self::Other { name, .. } => name.as_str(),
        }
    }

    /// Return the stored .app path, if any.
    pub fn app_path(&self) -> Option<&str> {
        match self {
            Self::Chrome { app_path }
            | Self::Firefox { app_path }
            | Self::Brave { app_path }
            | Self::Edge { app_path }
            | Self::Safari { app_path }
            | Self::Arc { app_path }
            | Self::Orion { app_path }
            | Self::Other { app_path, .. } => app_path.as_deref(),
        }
    }

    /// Return a new Browser with the given .app path.
    pub fn with_app_path(&self, path: String) -> Self {
        let set = |app_path: &mut Option<String>| *app_path = Some(path);
        let mut b = self.clone();
        match &mut b {
            Self::Chrome { app_path }
            | Self::Firefox { app_path }
            | Self::Brave { app_path }
            | Self::Edge { app_path }
            | Self::Safari { app_path }
            | Self::Arc { app_path }
            | Self::Orion { app_path }
            | Self::Other { app_path, .. } => set(app_path),
        }
        b
    }

    pub fn bundle_id(&self) -> &str {
        match self {
            Self::Chrome { .. } => "com.google.Chrome",
            Self::Firefox { .. } => "org.mozilla.firefox",
            Self::Brave { .. } => "com.brave.Browser",
            Self::Edge { .. } => "com.microsoft.Edge",
            Self::Safari { .. } => "com.apple.Safari",
            Self::Arc { .. } => "company.thebrowser.Browser",
            Self::Orion { .. } => "com.kagi.Orion",
            Self::Other { .. } => "",
        }
    }

    pub fn exec_name(&self) -> &str {
        match self {
            Self::Chrome { .. } => "Google Chrome",
            Self::Firefox { .. } => "Firefox",
            Self::Brave { .. } => "Brave Browser",
            Self::Edge { .. } => "Microsoft Edge",
            Self::Safari { .. } => "Safari",
            Self::Arc { .. } => "Arc",
            Self::Orion { .. } => "Orion",
            Self::Other { name, .. } => name.as_str(),
        }
    }

    fn variant_name(&self) -> &str {
        match self {
            Self::Chrome { .. } => "Chrome",
            Self::Firefox { .. } => "Firefox",
            Self::Brave { .. } => "Brave",
            Self::Edge { .. } => "Edge",
            Self::Safari { .. } => "Safari",
            Self::Arc { .. } => "Arc",
            Self::Orion { .. } => "Orion",
            Self::Other { .. } => "Other",
        }
    }
}

// ---------------------------------------------------------------------------
// Custom Serialize / Deserialize for backward-compatible YAML config
//
// Formats accepted (all YAML):
//   - "Chrome"                    → known browser, no path
//   - {"Chrome": "/path"}         → known browser with path
//   - {"name": "X", "app_path": "/p"}  → Other browser (flat, new format)
//   - {"Other": {"name": "X", ...}}   → Other browser (nested, old format)
//
// Serialization always uses the compact form.
// ---------------------------------------------------------------------------

impl Serialize for Browser {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        match self {
            // Known browsers without path: serialize as plain string
            Self::Chrome { app_path: None }
            | Self::Firefox { app_path: None }
            | Self::Brave { app_path: None }
            | Self::Edge { app_path: None }
            | Self::Safari { app_path: None }
            | Self::Arc { app_path: None }
            | Self::Orion { app_path: None } => {
                return serializer.serialize_str(self.variant_name())
            }
            // Known browsers with path: serialize as {"Name": "/path"}
            Self::Chrome { app_path: Some(p) } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Chrome", p)?;
                return map.end();
            }
            Self::Firefox { app_path: Some(p) } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Firefox", p)?;
                return map.end();
            }
            Self::Brave { app_path: Some(p) } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Brave", p)?;
                return map.end();
            }
            Self::Edge { app_path: Some(p) } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Edge", p)?;
                return map.end();
            }
            Self::Safari { app_path: Some(p) } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Safari", p)?;
                return map.end();
            }
            Self::Arc { app_path: Some(p) } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Arc", p)?;
                return map.end();
            }
            Self::Orion { app_path: Some(p) } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Orion", p)?;
                return map.end();
            }
            // Other: serialize flat {"name": ..., "app_path": ...}
            Self::Other { name, app_path } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("name", name)?;
                map.serialize_entry("app_path", app_path)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Browser {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(BrowserVisitor)
    }
}

struct BrowserVisitor;

impl<'de> Visitor<'de> for BrowserVisitor {
    type Value = Browser;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a browser name string or a map")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Browser, E> {
        match v {
            "Chrome" => Ok(Browser::Chrome { app_path: None }),
            "Firefox" => Ok(Browser::Firefox { app_path: None }),
            "Brave" => Ok(Browser::Brave { app_path: None }),
            "Edge" => Ok(Browser::Edge { app_path: None }),
            "Safari" => Ok(Browser::Safari { app_path: None }),
            "Arc" => Ok(Browser::Arc { app_path: None }),
            "Orion" => Ok(Browser::Orion { app_path: None }),
            other => Ok(Browser::Other {
                name: other.to_string(),
                app_path: None,
            }),
        }
    }

    fn visit_map<M: de::MapAccess<'de>>(self, mut map: M) -> Result<Browser, M::Error> {
        use serde::de::Error;

        // Collect all entries into a Vec<(String, serde_yaml::Value)> so we can
        // inspect the structure without being constrained by visitor ordering.
        let mut entries: Vec<(String, serde_yaml::Value)> = Vec::new();
        while let Some((k, v)) = map.next_entry::<String, serde_yaml::Value>()? {
            entries.push((k, v));
        }

        if entries.is_empty() {
            return Err(M::Error::custom("expected a non-empty map for browser"));
        }

        // ── Single entry → known browser with path, or old Other format ──
        if entries.len() == 1 {
            let (k, v) = entries.into_iter().next().unwrap();
            match k.as_str() {
                "Chrome" => {
                    let path = v.as_str().map(|s| s.to_string());
                    return Ok(Browser::Chrome { app_path: path });
                }
                "Firefox" => {
                    let path = v.as_str().map(|s| s.to_string());
                    return Ok(Browser::Firefox { app_path: path });
                }
                "Brave" => {
                    let path = v.as_str().map(|s| s.to_string());
                    return Ok(Browser::Brave { app_path: path });
                }
                "Edge" => {
                    let path = v.as_str().map(|s| s.to_string());
                    return Ok(Browser::Edge { app_path: path });
                }
                "Safari" => {
                    let path = v.as_str().map(|s| s.to_string());
                    return Ok(Browser::Safari { app_path: path });
                }
                "Arc" => {
                    let path = v.as_str().map(|s| s.to_string());
                    return Ok(Browser::Arc { app_path: path });
                }
                "Orion" => {
                    let path = v.as_str().map(|s| s.to_string());
                    return Ok(Browser::Orion { app_path: path });
                }
                "Other" => {
                    // Old nested format: {"Other": {"name": "X", "app_path": "..."}}
                    if let Some(inner) = v.as_mapping() {
                        let name = inner
                            .get(&serde_yaml::Value::String("name".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_default();
                        let app_path = inner
                            .get(&serde_yaml::Value::String("app_path".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        return Ok(Browser::Other { name, app_path });
                    }
                    // Fallback: "Other" with direct string value
                    let name = v.as_str().unwrap_or("Other");
                    return Ok(Browser::Other {
                        name: name.to_string(),
                        app_path: None,
                    });
                }
                _ => {
                    // Single unknown key → treat as custom browser with path
                    let path = v.as_str().map(|s| s.to_string());
                    return Ok(Browser::Other {
                        name: k,
                        app_path: path,
                    });
                }
            }
        }

        // ── Multiple entries → flat Other format ──
        // { "name": "X", "app_path": "/path" }
        let mut name = None;
        let mut app_path = None;
        for (k, v) in entries {
            match k.as_str() {
                "name" => name = v.as_str().map(|s| s.to_string()),
                "app_path" => app_path = v.as_str().map(|s| s.to_string()),
                _ => {}
            }
        }
        let name = name.ok_or_else(|| M::Error::custom("missing 'name' in browser object"))?;
        Ok(Browser::Other { name, app_path })
    }
}
