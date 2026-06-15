use serde::{Deserialize, Serialize};

use crate::browser::Browser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Exact domain to match (e.g., "github.com")
    pub domain: Option<String>,
    /// Glob or regex pattern for URL matching
    pub pattern: Option<String>,
    pub browser: Browser,
    pub profile: Option<String>,
}

impl Rule {
    pub fn matches(&self, url: &url::Url) -> bool {
        if let Some(domain) = &self.domain {
            if let Some(host) = url.host_str() {
                if host == domain || host.ends_with(&format!(".{}", domain)) {
                    return true;
                }
            }
        }
        if let Some(pattern) = &self.pattern {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if re.is_match(url.as_str()) {
                    return true;
                }
            }
        }
        false
    }
}

/// Evaluate all rules and find the first match.
pub fn evaluate_rules<'a>(rules: &'a [Rule], url: &url::Url) -> Option<&'a Rule> {
    rules.iter().find(|r| r.matches(url))
}
