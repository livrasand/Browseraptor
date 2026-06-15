pub mod detector;
pub mod launcher;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Browser {
    Chrome,
    Firefox,
    Brave,
    Edge,
    Safari,
    Arc,
    Orion,
    Other {
        name: String,
        app_path: Option<String>,
    },
}

impl Browser {
    pub fn name(&self) -> &str {
        match self {
            Self::Chrome => "Chrome",
            Self::Firefox => "Firefox",
            Self::Brave => "Brave",
            Self::Edge => "Edge",
            Self::Safari => "Safari",
            Self::Arc => "Arc",
            Self::Orion => "Orion",
            Self::Other { name, .. } => name.as_str(),
        }
    }

    pub fn bundle_id(&self) -> &str {
        match self {
            Self::Chrome => "com.google.Chrome",
            Self::Firefox => "org.mozilla.firefox",
            Self::Brave => "com.brave.Browser",
            Self::Edge => "com.microsoft.Edge",
            Self::Safari => "com.apple.Safari",
            Self::Arc => "company.thebrowser.Browser",
            Self::Orion => "com.kagi.Orion",
            Self::Other { .. } => "",
        }
    }

    pub fn exec_name(&self) -> &str {
        match self {
            Self::Chrome => "Google Chrome",
            Self::Firefox => "Firefox",
            Self::Brave => "Brave Browser",
            Self::Edge => "Microsoft Edge",
            Self::Safari => "Safari",
            Self::Arc => "Arc",
            Self::Orion => "Orion",
            Self::Other { name, .. } => name.as_str(),
        }
    }
}
