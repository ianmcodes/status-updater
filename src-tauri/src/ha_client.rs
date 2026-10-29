use std::time::Duration;
use crate::models::{EntityState, EntitySummary, HaRawState};

pub fn normalize_url(raw: &str) -> String {
    let mut url = raw.trim().to_string();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("http://{}", url);
    }
    while url.ends_with('/') {
        url.pop();
    }
    url
}

pub async fn fetch_entity(
    client: &reqwest::Client,
    ha_url: &str,
    token: &str,
    entity_id: &str,
) -> Result<EntityState, String> {
    if ha_url.trim().is_empty() {
        return Err("Home Assistant URL cannot be empty.".to_string());
    }
    if token.trim().is_empty() {
        return Err("Home Assistant Access Token cannot be empty.".to_string());
    }
    let trimmed_entity = entity_id.trim();
    if trimmed_entity.is_empty() {
        return Err("Entity ID cannot be empty.".to_string());
    }

    let base = normalize_url(ha_url);
    let endpoint = format!("{}/api/states/{}", base, trimmed_entity);

    let res = client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {}", token.trim()))
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Network error connecting to Home Assistant: {}", e))?;

    let status = res.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Unauthorized (401): Invalid Long-Lived Access Token. Please verify your token in Home Assistant (Profile > Security > Long-Lived Access Tokens).".to_string());
    }

    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(format!("Entity '{}' not found (404) in Home Assistant. Verify the entity ID exists under Developer Tools > States.", trimmed_entity));
    }

    if !status.is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Home Assistant returned HTTP {}: {}", status, text));
    }

    let raw: HaRawState = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse entity JSON response: {}", e))?;

    let state: EntityState = raw.into();

    if state.options.is_empty() {
        return Err(format!(
            "Entity '{}' was found, but it has no options listed. Is it an 'input_select' entity?",
            trimmed_entity
        ));
    }

    Ok(state)
}

pub async fn set_entity_option(
    client: &reqwest::Client,
    ha_url: &str,
    token: &str,
    entity_id: &str,
    option: &str,
) -> Result<EntityState, String> {
    let base = normalize_url(ha_url);
    let endpoint = format!("{}/api/services/input_select/select_option", base);

    let payload = serde_json::json!({
        "entity_id": entity_id.trim(),
        "option": option.trim()
    });

    let res = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", token.trim()))
        .header("Content-Type", "application/json")
        .json(&payload)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to send update request: {}", e))?;

    let status = res.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Unauthorized (401): Invalid Long-Lived Access Token.".to_string());
    }
    if !status.is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Failed to set option (HTTP {}): {}", status, text));
    }

    // Try to parse returned updated state array
    if let Ok(raw_list) = res.json::<Vec<HaRawState>>().await {
        for raw in raw_list {
            if raw.entity_id == entity_id.trim() {
                let state: EntityState = raw.into();
                return Ok(state);
            }
        }
    }

    // Fallback: re-fetch entity state directly
    fetch_entity(client, ha_url, token, entity_id).await
}

pub async fn discover_input_selects(
    client: &reqwest::Client,
    ha_url: &str,
    token: &str,
) -> Result<Vec<EntitySummary>, String> {
    let base = normalize_url(ha_url);
    let endpoint = format!("{}/api/states", base);

    let res = client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {}", token.trim()))
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Home Assistant: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Failed to discover entities (HTTP {}): {}", status, text));
    }

    let all_states: Vec<HaRawState> = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse states list: {}", e))?;

    let mut list = Vec::new();
    for raw in all_states {
        if raw.entity_id.starts_with("input_select.") {
            let options_count = raw.attributes.as_ref().map(|a| a.options.len()).unwrap_or(0);
            let friendly_name = raw.attributes.as_ref().and_then(|a| a.friendly_name.clone());
            list.push(EntitySummary {
                entity_id: raw.entity_id,
                friendly_name,
                current_state: raw.state,
                options_count,
            });
        }
    }

    list.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("http://homeassistant.local:8123"), "http://homeassistant.local:8123");
        assert_eq!(normalize_url("http://homeassistant.local:8123/"), "http://homeassistant.local:8123");
        assert_eq!(normalize_url("homeassistant.local:8123///"), "http://homeassistant.local:8123");
        assert_eq!(normalize_url("https://ha.mydomain.com/"), "https://ha.mydomain.com");
        assert_eq!(normalize_url("  192.168.1.10:8123  "), "http://192.168.1.10:8123");
    }

    #[test]
    fn test_parse_input_select_raw_state() {
        let json_data = r#"{
            "entity_id": "input_select.house_mode",
            "state": "Home",
            "attributes": {
                "options": ["Home", "Away", "Night", "Vacation"],
                "friendly_name": "House Mode",
                "icon": "mdi:home"
            },
            "last_updated": "2026-10-29T05:00:00Z"
        }"#;

        let raw: HaRawState = serde_json::from_str(json_data).expect("Failed to deserialize JSON");
        let state: EntityState = raw.into();

        assert_eq!(state.entity_id, "input_select.house_mode");
        assert_eq!(state.state, "Home");
        assert_eq!(state.friendly_name.as_deref(), Some("House Mode"));
        assert_eq!(state.options, vec!["Home", "Away", "Night", "Vacation"]);
        assert_eq!(state.icon.as_deref(), Some("mdi:home"));
    }
}

