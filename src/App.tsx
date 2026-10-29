import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Server,
  Key,
  Tag,
  Check,
  CheckCircle2,
  AlertCircle,
  RefreshCw,
  Sliders,
  Eye,
  EyeOff,
  Save,
  Radio,
  FileKey,
  Layers,
  Minimize2,
  ListFilter
} from "./icons";
import { AppConfig, EntityState, EntitySummary } from "./types";
import "./App.css";

export default function App() {
  const [config, setConfig] = useState<AppConfig>({
    ha_url: "http://homeassistant.local:8123",
    entity_id: "",
    access_token: "",
    refresh_interval_secs: 60,
    start_minimized: false,
  });

  const [entityState, setEntityState] = useState<EntityState | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [refreshing, setRefreshing] = useState<boolean>(false);
  const [saving, setSaving] = useState<boolean>(false);
  const [testing, setTesting] = useState<boolean>(false);
  const [discovering, setDiscovering] = useState<boolean>(false);
  const [discoveredList, setDiscoveredList] = useState<EntitySummary[]>([]);
  const [changingOption, setChangingOption] = useState<string | null>(null);
  const [showToken, setShowToken] = useState<boolean>(false);
  const [hasCredentialsToken, setHasCredentialsToken] = useState<boolean>(false);
  const [feedback, setFeedback] = useState<{
    type: "success" | "error" | "info";
    message: string;
  } | null>(null);

  // Load initial settings and HA state on mount
  useEffect(() => {
    let unlistenChange: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;

    async function init() {
      try {
        const loadedConfig = await invoke<AppConfig>("get_config");
        setConfig(loadedConfig);

        // Check if credentials.json is present
        try {
          const credToken = await invoke<string>("load_token_from_credentials");
          if (credToken) {
            setHasCredentialsToken(true);
            if (!loadedConfig.access_token) {
              setConfig((prev) => ({ ...prev, access_token: credToken }));
            }
          }
        } catch {
          // No credentials file, ignore
        }

        // Fetch state if credentials and entity are set
        if (loadedConfig.entity_id && loadedConfig.access_token) {
          try {
            const state = await invoke<EntityState>("fetch_ha_state");
            setEntityState(state);
          } catch (err: any) {
            setFeedback({ type: "error", message: String(err) });
          }
        }
      } catch (err: any) {
        setFeedback({ type: "error", message: `Init error: ${err}` });
      } finally {
        setLoading(false);
      }

      // Listen for tray state changes or background polling updates
      unlistenChange = await listen<EntityState>("ha-state-changed", (event) => {
        setEntityState(event.payload);
        setFeedback(null);
      });

      unlistenError = await listen<string>("ha-error", (event) => {
        setFeedback({ type: "error", message: event.payload });
      });
    }

    init();

    return () => {
      if (unlistenChange) unlistenChange();
      if (unlistenError) unlistenError();
    };
  }, []);

  async function handleRefresh() {
    setRefreshing(true);
    setFeedback(null);
    try {
      const state = await invoke<EntityState>("fetch_ha_state");
      setEntityState(state);
      setFeedback({ type: "success", message: "State successfully refreshed." });
    } catch (err: any) {
      setFeedback({ type: "error", message: String(err) });
    } finally {
      setRefreshing(false);
    }
  }

  async function handleSelectOption(option: string) {
    if (changingOption || entityState?.state === option) return;
    setChangingOption(option);
    setFeedback(null);
    try {
      const updated = await invoke<EntityState>("set_ha_option", { option });
      setEntityState(updated);
      setFeedback({
        type: "success",
        message: `State updated to "${option}".`,
      });
    } catch (err: any) {
      setFeedback({ type: "error", message: `Failed to set state: ${err}` });
    } finally {
      setChangingOption(null);
    }
  }

  async function handleTestConnection() {
    setTesting(true);
    setFeedback(null);
    try {
      const testRes = await invoke<EntityState>("test_connection", {
        haUrl: config.ha_url,
        accessToken: config.access_token,
        entityId: config.entity_id,
      });
      setEntityState(testRes);
      setFeedback({
        type: "success",
        message: `Connection successful! Found "${testRes.friendly_name || testRes.entity_id}" with ${testRes.options.length} options. Current state is "${testRes.state}".`,
      });
    } catch (err: any) {
      setFeedback({
        type: "error",
        message: `Connection test failed: ${err}`,
      });
    } finally {
      setTesting(false);
    }
  }

  async function handleSaveConfig(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setFeedback(null);
    try {
      const state = await invoke<EntityState>("save_config", {
        newConfig: config,
      });
      setEntityState(state);
      setFeedback({
        type: "success",
        message: "Settings saved! System tray menu and state updated.",
      });
    } catch (err: any) {
      setFeedback({
        type: "error",
        message: `Saved settings, but failed to fetch entity: ${err}`,
      });
    } finally {
      setSaving(false);
    }
  }

  async function handleDiscoverEntities() {
    if (!config.ha_url || !config.access_token) {
      setFeedback({
        type: "error",
        message: "Please enter Home Assistant URL and Access Token first.",
      });
      return;
    }
    setDiscovering(true);
    setFeedback(null);
    try {
      const list = await invoke<EntitySummary[]>("discover_input_selects", {
        haUrl: config.ha_url,
        accessToken: config.access_token,
      });
      setDiscoveredList(list);
      if (list.length === 0) {
        setFeedback({
          type: "info",
          message: "No input_select entities found in your Home Assistant instance.",
        });
      } else {
        setFeedback({
          type: "success",
          message: `Found ${list.length} input_select entity(ies). Select one below.`,
        });
      }
    } catch (err: any) {
      setFeedback({
        type: "error",
        message: `Failed to detect entities: ${err}`,
      });
    } finally {
      setDiscovering(false);
    }
  }

  async function handleImportCredentials() {
    try {
      const token = await invoke<string>("load_token_from_credentials");
      setConfig((prev) => ({ ...prev, access_token: token }));
      setFeedback({
        type: "success",
        message: "Loaded access token from credentials.json.",
      });
    } catch (err: any) {
      setFeedback({
        type: "error",
        message: `Could not load credentials: ${err}`,
      });
    }
  }

  async function handleMinimizeToTray() {
    try {
      await invoke("hide_window");
    } catch (err: any) {
      console.error(err);
    }
  }

  if (loading) {
    return (
      <div className="loading-screen">
        <RefreshCw className="spinner" size={32} />
        <p>Loading Home Assistant Status Updater...</p>
      </div>
    );
  }

  return (
    <div className="app-container">
      {/* Top Header */}
      <header className="header">
        <div className="header-title">
          <div className="icon-badge">
            <Radio size={20} className="header-icon" />
          </div>
          <div>
            <h1>HA Status Updater</h1>
            <p className="subtitle">Cross-platform desktop & tray controller</p>
          </div>
        </div>

        <div className="header-actions">
          <button
            type="button"
            className="btn-tray-hide"
            onClick={handleMinimizeToTray}
            title="Minimize to System Tray"
          >
            <Minimize2 size={16} />
            <span>Hide to Tray</span>
          </button>
        </div>
      </header>

      {/* Notification banner */}
      {feedback && (
        <div className={`feedback-alert alert-${feedback.type}`}>
          {feedback.type === "success" && <CheckCircle2 size={18} />}
          {feedback.type === "error" && <AlertCircle size={18} />}
          {feedback.type === "info" && <Sliders size={18} />}
          <div className="feedback-text">{feedback.message}</div>
          <button
            type="button"
            className="alert-close"
            onClick={() => setFeedback(null)}
          >
            ✕
          </button>
        </div>
      )}

      {/* Main Content Area */}
      <main className="content">
        {/* Current State & Quick Options Section */}
        <section className="card state-card">
          <div className="card-header">
            <div className="card-title">
              <Layers size={18} />
              <h2>Current Entity State</h2>
            </div>
            <button
              type="button"
              className="btn-icon"
              onClick={handleRefresh}
              disabled={refreshing || !config.entity_id}
              title="Refresh State from Home Assistant"
            >
              <RefreshCw size={15} className={refreshing ? "spinner" : ""} />
              <span>Refresh</span>
            </button>
          </div>

          {entityState ? (
            <div className="entity-view">
              <div className="entity-meta">
                <div className="meta-left">
                  <span className="entity-name">
                    {entityState.friendly_name || entityState.entity_id}
                  </span>
                  <span className="entity-badge">{entityState.entity_id}</span>
                </div>
                <div className="current-state-pill">
                  <span className="dot pulse"></span>
                  <span className="state-label">{entityState.state}</span>
                </div>
              </div>

              <div className="options-container">
                <span className="options-title">Select State Option:</span>
                <div className="options-grid">
                  {entityState.options.map((opt) => {
                    const isSelected = entityState.state === opt;
                    const isActivating = changingOption === opt;
                    return (
                      <button
                        key={opt}
                        type="button"
                        className={`option-btn ${isSelected ? "selected" : ""}`}
                        disabled={changingOption !== null}
                        onClick={() => handleSelectOption(opt)}
                      >
                        {isActivating ? (
                          <RefreshCw size={14} className="spinner" />
                        ) : isSelected ? (
                          <Check size={14} className="check-icon" />
                        ) : (
                          <span className="circle-placeholder"></span>
                        )}
                        <span>{opt}</span>
                      </button>
                    );
                  })}
                </div>
              </div>

              {entityState.last_updated && (
                <div className="last-updated">
                  Last updated:{" "}
                  {new Date(entityState.last_updated).toLocaleTimeString()}
                </div>
              )}
            </div>
          ) : (
            <div className="empty-state">
              <AlertCircle size={24} className="empty-icon" />
              <p>No entity state loaded yet.</p>
              <small>Configure your Home Assistant details below to connect.</small>
            </div>
          )}
        </section>

        {/* Configuration Form */}
        <section className="card config-card">
          <div className="card-header">
            <div className="card-title">
              <Sliders size={18} />
              <h2>Configuration</h2>
            </div>
            {hasCredentialsToken && (
              <button
                type="button"
                className="btn-subtle"
                onClick={handleImportCredentials}
                title="Fill token from credentials.json"
              >
                <FileKey size={14} />
                <span>credentials.json found</span>
              </button>
            )}
          </div>

          <form onSubmit={handleSaveConfig} className="config-form">
            {/* Home Assistant URL */}
            <div className="form-group">
              <label htmlFor="ha_url">
                <Server size={15} />
                <span>Home Assistant Base URL</span>
              </label>
              <input
                id="ha_url"
                type="text"
                value={config.ha_url}
                onChange={(e) =>
                  setConfig({ ...config, ha_url: e.target.value })
                }
                placeholder="http://homeassistant.local:8123 or https://..."
                required
              />
              <span className="input-hint">
                Local IP, http://homeassistant.local:8123, or remote URL.
              </span>
            </div>

            {/* Entity ID */}
            <div className="form-group">
              <div className="label-with-action">
                <label htmlFor="entity_id">
                  <Tag size={15} />
                  <span>Entity ID (input_select.*)</span>
                </label>
                <button
                  type="button"
                  className="btn-link"
                  onClick={handleDiscoverEntities}
                  disabled={discovering}
                >
                  <ListFilter size={13} />
                  <span>{discovering ? "Detecting..." : "Detect Entities"}</span>
                </button>
              </div>

              <input
                id="entity_id"
                type="text"
                value={config.entity_id}
                onChange={(e) =>
                  setConfig({ ...config, entity_id: e.target.value })
                }
                placeholder="e.g. input_select.house_mode"
                required
              />

              {/* Entity Picker if discovered */}
              {discoveredList.length > 0 && (
                <div className="discovered-dropdown">
                  <label className="dropdown-label">Detected Entities:</label>
                  <select
                    className="entity-select"
                    value={config.entity_id}
                    onChange={(e) =>
                      setConfig({ ...config, entity_id: e.target.value })
                    }
                  >
                    <option value="">-- Choose from discovered entities --</option>
                    {discoveredList.map((ent) => (
                      <option key={ent.entity_id} value={ent.entity_id}>
                        {ent.friendly_name
                          ? `${ent.friendly_name} (${ent.entity_id}) - [${ent.current_state}]`
                          : `${ent.entity_id} - [${ent.current_state}]`}
                      </option>
                    ))}
                  </select>
                </div>
              )}
            </div>

            {/* Long-Lived Access Token */}
            <div className="form-group">
              <label htmlFor="access_token">
                <Key size={15} />
                <span>Long-Lived Access Token</span>
              </label>
              <div className="input-password-wrapper">
                <input
                  id="access_token"
                  type={showToken ? "text" : "password"}
                  value={config.access_token}
                  onChange={(e) =>
                    setConfig({ ...config, access_token: e.target.value })
                  }
                  placeholder="Paste your token from Home Assistant"
                  required
                />
                <button
                  type="button"
                  className="btn-toggle-eye"
                  onClick={() => setShowToken(!showToken)}
                  title={showToken ? "Hide token" : "Show token"}
                >
                  {showToken ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
              <span className="input-hint">
                Create in HA: Click your Profile &gt; Security &gt; Long-Lived Access Tokens.
              </span>
            </div>

            {/* Polling Interval & Behavior */}
            <div className="form-row">
              <div className="form-group half">
                <label htmlFor="refresh_interval">
                  <RefreshCw size={15} />
                  <span>Tray Sync Interval</span>
                </label>
                <select
                  id="refresh_interval"
                  value={config.refresh_interval_secs}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      refresh_interval_secs: Number(e.target.value),
                    })
                  }
                >
                  <option value={15}>Every 15 seconds</option>
                  <option value={30}>Every 30 seconds</option>
                  <option value={60}>Every 1 minute</option>
                  <option value={120}>Every 2 minutes</option>
                  <option value={300}>Every 5 minutes</option>
                  <option value={0}>Manual Only (0s)</option>
                </select>
              </div>

              <div className="form-group half checkbox-group">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={config.start_minimized}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        start_minimized: e.target.checked,
                      })
                    }
                  />
                  <span>Start minimized to tray</span>
                </label>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="form-actions">
              <button
                type="button"
                className="btn btn-secondary"
                onClick={handleTestConnection}
                disabled={testing || !config.ha_url || !config.access_token}
              >
                {testing ? (
                  <RefreshCw size={16} className="spinner" />
                ) : (
                  <CheckCircle2 size={16} />
                )}
                <span>{testing ? "Testing..." : "Test Connection"}</span>
              </button>

              <button
                type="submit"
                className="btn btn-primary"
                disabled={saving}
              >
                {saving ? (
                  <RefreshCw size={16} className="spinner" />
                ) : (
                  <Save size={16} />
                )}
                <span>{saving ? "Saving..." : "Save Configuration"}</span>
              </button>
            </div>
          </form>
        </section>

        {/* System Tray Info Banner */}
        <section className="tray-info-banner">
          <div className="tray-icon-preview">
            <Radio size={20} />
          </div>
          <div className="tray-info-text">
            <strong>System Tray Menu Active</strong>
            <p>
              Click the tray icon anytime to see your current state and switch
              options directly without opening this window. Closing this window hides it to the system tray.
            </p>
          </div>
        </section>
      </main>
    </div>
  );
}
