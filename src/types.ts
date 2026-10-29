export interface AppConfig {
  ha_url: string;
  entity_id: string;
  access_token: string;
  refresh_interval_secs: number;
  start_minimized: boolean;
}

export interface EntityState {
  entity_id: string;
  state: string;
  friendly_name?: string | null;
  options: string[];
  last_updated?: string | null;
  icon?: string | null;
}

export interface EntitySummary {
  entity_id: string;
  friendly_name?: string | null;
  current_state: string;
  options_count: number;
}
