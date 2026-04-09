use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Tone
// ---------------------------------------------------------------------------

/// Severity/tone level for blame messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tone {
    Gentle,
    Firm,
    PassiveAggressive,
    ScorchedEarth,
}

impl Tone {
    /// Parse a tone from a string (case-insensitive, accepts kebab-case).
    pub fn from_str(s: &str) -> Option<Tone> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "gentle" => Some(Tone::Gentle),
            "firm" => Some(Tone::Firm),
            "passive-aggressive" => Some(Tone::PassiveAggressive),
            "scorched-earth" => Some(Tone::ScorchedEarth),
            _ => None,
        }
    }
}

impl Default for Tone {
    fn default() -> Self {
        Tone::PassiveAggressive
    }
}

impl fmt::Display for Tone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tone::Gentle => write!(f, "gentle"),
            Tone::Firm => write!(f, "firm"),
            Tone::PassiveAggressive => write!(f, "passive-aggressive"),
            Tone::ScorchedEarth => write!(f, "scorched-earth"),
        }
    }
}

// ---------------------------------------------------------------------------
// Config structs
// ---------------------------------------------------------------------------

/// Top-level `.gitblame` configuration.
#[derive(Debug, Clone, Default)]
pub struct BlameConfig {
    /// General project-wide settings.
    pub general: GeneralConfig,
    /// Per-violation severity overrides.
    pub severity: SeverityConfig,
}

/// Default OpenRouter model for Sophisticated AI™.
pub const DEFAULT_MODEL: &str = "google/gemma-4-31b-it:free";

/// General configuration section.
#[derive(Debug, Clone)]
pub struct GeneralConfig {
    /// Default tone for blame messages.
    pub tone: Tone,
    /// Additional CC recipients for every blame email.
    pub cc: Vec<String>,
    /// Optional group email for CC.
    pub cc_group: Option<String>,
    /// Number of offenses before escalation.
    pub escalation_threshold: u32,
    /// OpenRouter model identifier for Sophisticated AI™.
    pub model: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            tone: Tone::PassiveAggressive,
            cc: Vec::new(),
            cc_group: None,
            escalation_threshold: 3,
            model: DEFAULT_MODEL.to_string(),
        }
    }
}

/// Mapping from violation name to severity tone.
pub type SeverityConfig = HashMap<String, Tone>;

// ---------------------------------------------------------------------------
// TOML deserialization helpers (private)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct RawConfig {
    general: Option<RawGeneral>,
    severity: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Default)]
struct RawGeneral {
    tone: Option<String>,
    cc: Option<Vec<String>>,
    cc_group: Option<String>,
    escalation_threshold: Option<u32>,
    model: Option<String>,
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

impl BlameConfig {
    /// Load the `.gitblame` configuration file.
    ///
    /// Searches for `.gitblame` starting in the current directory and walking
    /// up to the filesystem root.  If the file is not found or cannot be
    /// parsed, a default configuration is returned — this method never errors.
    pub fn load() -> Self {
        // Honor GITBLAME_DOTGITBLAME as an explicit override path.
        if let Ok(path) = std::env::var("GITBLAME_DOTGITBLAME") {
            if !path.is_empty() {
                return Self::load_from_path(&PathBuf::from(path));
            }
        }
        match find_config_file() {
            Some(path) => Self::load_from_path(&path),
            None => Self::default(),
        }
    }

    /// Load from a specific path, falling back to defaults on any error.
    fn load_from_path(path: &PathBuf) -> Self {
        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        let raw: RawConfig = match toml::from_str(&contents) {
            Ok(r) => r,
            Err(_) => return Self::default(),
        };

        let general = match raw.general {
            Some(g) => GeneralConfig {
                tone: g
                    .tone
                    .as_deref()
                    .and_then(Tone::from_str)
                    .unwrap_or_default(),
                cc: g.cc.unwrap_or_default(),
                cc_group: g.cc_group,
                escalation_threshold: g.escalation_threshold.unwrap_or(3),
                model: g.model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            },
            None => GeneralConfig::default(),
        };

        let severity = match raw.severity {
            Some(map) => map
                .into_iter()
                .filter_map(|(k, v)| Tone::from_str(&v).map(|t| (k, t)))
                .collect(),
            None => HashMap::new(),
        };

        Self { general, severity }
    }
}

/// Walk from the current directory up to root looking for `.gitblame`.
fn find_config_file() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".gitblame");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_display_roundtrip() {
        for tone in [
            Tone::Gentle,
            Tone::Firm,
            Tone::PassiveAggressive,
            Tone::ScorchedEarth,
        ] {
            let s = tone.to_string();
            assert_eq!(Tone::from_str(&s), Some(tone));
        }
    }

    #[test]
    fn default_config_values() {
        let cfg = BlameConfig::default();
        assert_eq!(cfg.general.tone, Tone::PassiveAggressive);
        assert_eq!(cfg.general.escalation_threshold, 3);
        assert!(cfg.general.cc.is_empty());
        assert!(cfg.severity.is_empty());
    }

    #[test]
    fn parse_sample_toml() {
        let toml_str = r#"
[general]
tone = "firm"
cc = ["lead@org.com"]
cc_group = "team@org.com"
escalation_threshold = 5

[severity]
unused_import = "gentle"
force_push_to_main = "scorched-earth"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let g = raw.general.unwrap();
        assert_eq!(g.tone.as_deref(), Some("firm"));
        assert_eq!(g.cc.as_deref().unwrap().len(), 1);
        assert_eq!(g.escalation_threshold, Some(5));

        let sev = raw.severity.unwrap();
        assert_eq!(sev.get("unused_import").unwrap(), "gentle");
        assert_eq!(sev.get("force_push_to_main").unwrap(), "scorched-earth");
    }

    #[test]
    fn load_from_temp_file_all_fields() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".gitblame");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            write!(
                f,
                r#"
[general]
tone = "scorched-earth"
cc = ["boss@corp.com", "hr@corp.com"]
cc_group = "all-hands@corp.com"
escalation_threshold = 10

[severity]
typo = "gentle"
data_loss = "scorched-earth"
bad_naming = "firm"
"#
            )
            .unwrap();
        }

        let cfg = BlameConfig::load_from_path(&path.to_path_buf());
        assert_eq!(cfg.general.tone, Tone::ScorchedEarth);
        assert_eq!(cfg.general.cc.len(), 2);
        assert_eq!(cfg.general.cc[0], "boss@corp.com");
        assert_eq!(cfg.general.cc_group.as_deref(), Some("all-hands@corp.com"));
        assert_eq!(cfg.general.escalation_threshold, 10);
        assert_eq!(cfg.severity.len(), 3);
        assert_eq!(cfg.severity.get("typo"), Some(&Tone::Gentle));
        assert_eq!(cfg.severity.get("data_loss"), Some(&Tone::ScorchedEarth));
        assert_eq!(cfg.severity.get("bad_naming"), Some(&Tone::Firm));
    }

    #[test]
    fn load_with_missing_fields_uses_defaults() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".gitblame");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            // Minimal config: only tone is set
            write!(f, "[general]\ntone = \"gentle\"\n").unwrap();
        }

        let cfg = BlameConfig::load_from_path(&path.to_path_buf());
        assert_eq!(cfg.general.tone, Tone::Gentle);
        // Defaults for missing fields
        assert!(cfg.general.cc.is_empty());
        assert!(cfg.general.cc_group.is_none());
        assert_eq!(cfg.general.escalation_threshold, 3);
        assert!(cfg.severity.is_empty());
    }

    #[test]
    fn load_empty_file_uses_defaults() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".gitblame");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            write!(f, "").unwrap();
        }

        let cfg = BlameConfig::load_from_path(&path.to_path_buf());
        assert_eq!(cfg.general.tone, Tone::PassiveAggressive);
        assert_eq!(cfg.general.escalation_threshold, 3);
    }

    #[test]
    fn load_malformed_toml_returns_defaults() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".gitblame");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            write!(f, "this is not valid {{{{ toml ]]]]").unwrap();
        }

        let cfg = BlameConfig::load_from_path(&path.to_path_buf());
        assert_eq!(cfg.general.tone, Tone::PassiveAggressive);
        assert_eq!(cfg.general.escalation_threshold, 3);
        assert!(cfg.general.cc.is_empty());
    }

    #[test]
    fn severity_mapping_parsing() {
        let toml_str = r#"
[severity]
unused_import = "gentle"
force_push = "scorched-earth"
missing_test = "firm"
unknown_tone = "nonexistent"
"#;
        let raw: RawConfig = toml::from_str(toml_str).unwrap();
        let severity: SeverityConfig = raw
            .severity
            .unwrap()
            .into_iter()
            .filter_map(|(k, v)| Tone::from_str(&v).map(|t| (k, t)))
            .collect();

        assert_eq!(severity.len(), 3, "unknown tone should be filtered out");
        assert_eq!(severity.get("unused_import"), Some(&Tone::Gentle));
        assert_eq!(severity.get("force_push"), Some(&Tone::ScorchedEarth));
        assert_eq!(severity.get("missing_test"), Some(&Tone::Firm));
        assert!(severity.get("unknown_tone").is_none());
    }

    #[test]
    fn tone_from_str_uppercase() {
        assert_eq!(Tone::from_str("GENTLE"), Some(Tone::Gentle));
        assert_eq!(Tone::from_str("FIRM"), Some(Tone::Firm));
        assert_eq!(Tone::from_str("PASSIVE-AGGRESSIVE"), Some(Tone::PassiveAggressive));
        assert_eq!(Tone::from_str("SCORCHED-EARTH"), Some(Tone::ScorchedEarth));
    }

    #[test]
    fn tone_from_str_mixed_case() {
        assert_eq!(Tone::from_str("Gentle"), Some(Tone::Gentle));
        assert_eq!(Tone::from_str("Firm"), Some(Tone::Firm));
        assert_eq!(Tone::from_str("Passive-Aggressive"), Some(Tone::PassiveAggressive));
    }

    #[test]
    fn tone_from_str_underscores() {
        // Underscores should be treated as hyphens
        assert_eq!(Tone::from_str("passive_aggressive"), Some(Tone::PassiveAggressive));
        assert_eq!(Tone::from_str("scorched_earth"), Some(Tone::ScorchedEarth));
    }

    #[test]
    fn tone_from_str_invalid() {
        assert_eq!(Tone::from_str(""), None);
        assert_eq!(Tone::from_str("angry"), None);
        assert_eq!(Tone::from_str("super-gentle"), None);
    }

    #[test]
    fn tone_default_is_passive_aggressive() {
        assert_eq!(Tone::default(), Tone::PassiveAggressive);
    }
}
