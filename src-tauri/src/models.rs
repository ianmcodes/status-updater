use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    pub entity_id: String,
    pub state: String,
    pub friendly_name: Option<String>,
    pub options: Vec<String>,
    pub last_updated: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
