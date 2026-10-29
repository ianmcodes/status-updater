use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub ha_url: String,
    pub entity_id: String,
    pub access_token: String,
    pub refresh_interval_secs: u64,
    pub start_minimized: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ha_url: "http://homeassistant.local:8123".to_string(),
            entity_id: "".to_string(),
            access_token: "".to_string(),
            refresh_interval_secs: 60,
            start_minimized: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityState {
    pub entity_id: String,
    pub state: String,
    pub friendly_name: Option<String>,
    pub options: Vec<String>,
    pub last_updated: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntitySummary {
    pub entity_id: String,
    pub friendly_name: Option<String>,
    pub current_state: String,
    pub options_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HaRawStateAttributes {
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub friendly_name: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HaRawState {
    pub entity_id: String,
    pub state: String,
    #[serde(default)]
    pub attributes: Option<HaRawStateAttributes>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

impl From<HaRawState> for EntityState {
    fn from(raw: HaRawState) -> Self {
        let (friendly_name, options, icon) = match raw.attributes {
            Some(attr) => (attr.friendly_name, attr.options, attr.icon),
            None => (None, Vec::new(), None),
        };

        Self {
            entity_id: raw.entity_id,
            state: raw.state,
            friendly_name,
            options,
            last_updated: raw.last_updated,
            icon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.ha_url, "http://homeassistant.local:8123");
        assert_eq!(config.entity_id, "");
        assert_eq!(config.access_token, "");
        assert_eq!(config.refresh_interval_secs, 60);
        assert!(!config.start_minimized);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let original = AppConfig {
            ha_url: "https://my-ha.duckdns.org".to_string(),
            entity_id: "input_select.house_mode".to_string(),
            access_token: "secret_token_123".to_string(),
            refresh_interval_secs: 30,
            start_minimized: true,
        };

        let serialized = serde_json::to_string(&original).expect("serialize config");
        let deserialized: AppConfig = serde_json::from_str(&serialized).expect("deserialize config");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_entity_state_from_raw_complete() {
        let raw = HaRawState {
            entity_id: "input_select.mode".to_string(),
            state: "Away".to_string(),
            attributes: Some(HaRawStateAttributes {
                options: vec!["Home".into(), "Away".into(), "Night".into()],
                friendly_name: Some("Mode".into()),
                icon: Some("mdi:home".into()),
            }),
            last_updated: Some("2026-10-29T12:00:00Z".into()),
        };

        let entity: EntityState = raw.into();
        assert_eq!(entity.entity_id, "input_select.mode");
        assert_eq!(entity.state, "Away");
        assert_eq!(entity.friendly_name.as_deref(), Some("Mode"));
        assert_eq!(entity.options, vec!["Home", "Away", "Night"]);
        assert_eq!(entity.icon.as_deref(), Some("mdi:home"));
        assert_eq!(entity.last_updated.as_deref(), Some("2026-10-29T12:00:00Z"));
    }

    #[test]
    fn test_entity_state_from_raw_missing_attributes() {
        let raw = HaRawState {
            entity_id: "input_select.empty".to_string(),
            state: "unknown".to_string(),
            attributes: None,
            last_updated: None,
        };

        let entity: EntityState = raw.into();
        assert_eq!(entity.entity_id, "input_select.empty");
        assert_eq!(entity.state, "unknown");
        assert_eq!(entity.friendly_name, None);
        assert!(entity.options.is_empty());
        assert_eq!(entity.icon, None);
        assert_eq!(entity.last_updated, None);
    }

    #[test]
    fn test_entity_summary_serialization() {
        let summary = EntitySummary {
            entity_id: "input_select.status".into(),
            friendly_name: Some("Current Status".into()),
            current_state: "Active".into(),
            options_count: 4,
        };

        let json = serde_json::to_string(&summary).expect("serialize summary");
        let deserialized: EntitySummary = serde_json::from_str(&json).expect("deserialize summary");
        assert_eq!(summary, deserialized);
    }
}
