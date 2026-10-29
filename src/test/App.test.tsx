import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import App from "../App";
import { AppConfig, EntityState, EntitySummary } from "../types";

// Mock Tauri APIs
const mockInvoke = vi.fn();
const mockListen = vi.fn();
let registeredListeners: Record<string, (event: { payload: any }) => void> = {};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, callback: (event: { payload: any }) => void) => {
    mockListen(event, callback);
    registeredListeners[event] = callback;
    return Promise.resolve(() => {
      delete registeredListeners[event];
    });
  },
}));

const mockConfig: AppConfig = {
  ha_url: "http://homeassistant.local:8123",
  entity_id: "input_select.house_mode",
  access_token: "test_token_12345",
  refresh_interval_secs: 60,
  start_minimized: false,
  zoom_detection_enabled: false,
  zoom_call_status: "",
};

const mockEntityState: EntityState = {
  entity_id: "input_select.house_mode",
  state: "Home",
  friendly_name: "House Mode",
  options: ["Home", "Away", "Night", "Vacation"],
  last_updated: "2026-10-29T10:00:00Z",
};

const mockDiscoveredList: EntitySummary[] = [
  {
    entity_id: "input_select.alarm_status",
    friendly_name: "Alarm Status",
    current_state: "disarmed",
    options_count: 3,
  },
  {
    entity_id: "input_select.house_mode",
    friendly_name: "House Mode",
    current_state: "Home",
    options_count: 4,
  },
];

describe("App Component", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    registeredListeners = {};

    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      switch (cmd) {
        case "get_config":
          return Promise.resolve(mockConfig);
        case "load_token_from_credentials":
          return Promise.resolve("imported_token_from_file");
        case "fetch_ha_state":
          return Promise.resolve(mockEntityState);
        case "set_ha_option":
          return Promise.resolve({
            ...mockEntityState,
            state: args.option,
          });
        case "test_connection":
          return Promise.resolve(mockEntityState);
        case "save_config":
          return Promise.resolve(mockEntityState);
        case "discover_input_selects":
          return Promise.resolve(mockDiscoveredList);
        case "hide_window":
          return Promise.resolve();
        default:
          return Promise.resolve();
      }
    });
  });

  it("loads and displays entity state and configuration on startup", async () => {
    const { container } = render(<App />);

    expect(screen.getByText(/Loading Home Assistant Status Updater/i)).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText("House Mode")).toBeInTheDocument();
      expect(screen.getByText("input_select.house_mode")).toBeInTheDocument();
    });

    const statePill = container.querySelector(".state-label");
    expect(statePill).toHaveTextContent("Home");

    // Check that all options are rendered as buttons
    expect(screen.getByRole("button", { name: /Home/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Away/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Night/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Vacation/i })).toBeInTheDocument();

    // The selected button has the 'selected' class
    const homeBtn = screen.getByRole("button", { name: /Home/i });
    expect(homeBtn).toHaveClass("selected");

    // Check configuration inputs populated
    const urlInput = screen.getByLabelText(/Home Assistant Base URL/i) as HTMLInputElement;
    expect(urlInput.value).toBe("http://homeassistant.local:8123");

    const tokenInput = screen.getByLabelText(/Long-Lived Access Token/i) as HTMLInputElement;
    expect(tokenInput.value).toBe("test_token_12345");
  });

  it("selects an option and updates state", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("House Mode")).toBeInTheDocument();
    });

    const awayBtn = screen.getByRole("button", { name: /Away/i });
    fireEvent.click(awayBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_ha_option", { option: "Away" });
      expect(screen.getByText(/State updated to "Away"/i)).toBeInTheDocument();
    });
  });

  it("refreshes entity state when clicking Refresh button", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("House Mode")).toBeInTheDocument();
    });

    const refreshBtn = screen.getByRole("button", { name: /Refresh/i });
    fireEvent.click(refreshBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("fetch_ha_state");
      expect(screen.getByText(/State successfully refreshed/i)).toBeInTheDocument();
    });
  });

  it("toggles password visibility when clicking eye button", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText(/Long-Lived Access Token/i)).toBeInTheDocument();
    });

    const tokenInput = screen.getByLabelText(/Long-Lived Access Token/i) as HTMLInputElement;
    expect(tokenInput.type).toBe("password");

    const toggleBtn = screen.getByTitle(/Show token/i);
    fireEvent.click(toggleBtn);

    expect(tokenInput.type).toBe("text");

    const hideBtn = screen.getByTitle(/Hide token/i);
    fireEvent.click(hideBtn);

    expect(tokenInput.type).toBe("password");
  });

  it("shows Zoom call status choices only when Zoom detection is enabled", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText(/Set status while I am on a Zoom call/i)).toBeInTheDocument();
    });

    expect(screen.queryByLabelText(/Status to use during a Zoom call/i)).not.toBeInTheDocument();
    fireEvent.click(screen.getByLabelText(/Set status while I am on a Zoom call/i));

    const statusSelect = screen.getByLabelText(/Status to use during a Zoom call/i) as HTMLSelectElement;
    fireEvent.change(statusSelect, { target: { value: "Away" } });
    expect(statusSelect.value).toBe("Away");
  });

  it("discovers input_select entities and selects one from dropdown", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Detect Entities")).toBeInTheDocument();
    });

    const detectBtn = screen.getByText("Detect Entities");
    fireEvent.click(detectBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("discover_input_selects", {
        haUrl: mockConfig.ha_url,
        accessToken: mockConfig.access_token,
      });
      expect(screen.getByText(/Found 2 input_select entity\(ies\)/i)).toBeInTheDocument();
    });

    const select = screen.getByLabelText(/Detected Entities:/i) as HTMLSelectElement;
    expect(select).toBeInTheDocument();

    fireEvent.change(select, { target: { value: "input_select.alarm_status" } });

    const entityInput = screen.getByPlaceholderText("e.g. input_select.house_mode") as HTMLInputElement;
    expect(entityInput.value).toBe("input_select.alarm_status");
  });

  it("tests connection and handles success", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Test Connection")).toBeInTheDocument();
    });

    const testBtn = screen.getByText("Test Connection");
    fireEvent.click(testBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("test_connection", {
        haUrl: mockConfig.ha_url,
        accessToken: mockConfig.access_token,
        entityId: mockConfig.entity_id,
      });
      expect(screen.getByText(/Connection successful!/i)).toBeInTheDocument();
    });
  });

  it("tests connection and displays error banner when failed", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "test_connection") {
        return Promise.reject("Unauthorized (401): Invalid Long-Lived Access Token.");
      }
      if (cmd === "get_config") return Promise.resolve(mockConfig);
      if (cmd === "fetch_ha_state") return Promise.resolve(mockEntityState);
      return Promise.resolve();
    });

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Test Connection")).toBeInTheDocument();
    });

    const testBtn = screen.getByText("Test Connection");
    fireEvent.click(testBtn);

    await waitFor(() => {
      expect(screen.getByText(/Connection test failed: Unauthorized \(401\)/i)).toBeInTheDocument();
    });
  });

  it("submits the configuration form and saves settings", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Save Configuration")).toBeInTheDocument();
    });

    const saveBtn = screen.getByText("Save Configuration");
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("save_config", {
        newConfig: mockConfig,
      });
      expect(screen.getByText(/Settings saved!/i)).toBeInTheDocument();
    });
  });

  it("minimizes to tray when clicking Hide to Tray", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByTitle(/Minimize to System Tray/i)).toBeInTheDocument();
    });

    const hideBtn = screen.getByTitle(/Minimize to System Tray/i);
    fireEvent.click(hideBtn);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("hide_window");
    });
  });

  it("reacts to real-time 'ha-state-changed' events emitted by background task", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("House Mode")).toBeInTheDocument();
    });

    // Simulate event from system tray or background sync
    const changeListener = registeredListeners["ha-state-changed"];
    expect(changeListener).toBeDefined();

    act(() => {
      changeListener({
        payload: {
          ...mockEntityState,
          state: "Night",
        },
      });
    });

    await waitFor(() => {
      const nightBtn = screen.getByRole("button", { name: /Night/i });
      expect(nightBtn).toHaveClass("selected");
    });
  });

  it("displays error banner when 'ha-error' event is received", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("House Mode")).toBeInTheDocument();
    });

    const errorListener = registeredListeners["ha-error"];
    expect(errorListener).toBeDefined();

    act(() => {
      errorListener({
        payload: "Network connection lost to Home Assistant",
      });
    });

    await waitFor(() => {
      expect(screen.getByText("Network connection lost to Home Assistant")).toBeInTheDocument();
    });
  });
});
