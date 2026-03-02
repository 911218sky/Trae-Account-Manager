import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import { useAuth } from "../contexts/AuthContext";

interface SettingsProps {
  onToast?: (type: "success" | "error" | "warning" | "info", message: string) => void;
}

export function Settings({ onToast }: SettingsProps) {
  const [traeMachineId, setTraeMachineId] = useState<string>("");
  const [traeRefreshing, setTraeRefreshing] = useState(false);
  const [clearingTrae, setClearingTrae] = useState(false);
  const [traePath, setTraePath] = useState<string>("");
  const [traePathLoading, setTraePathLoading] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [forceReauthLoading, setForceReauthLoading] = useState(false);

  const { forceReauth } = useAuth();

  // Load Trae IDE machine ID
  const loadTraeMachineId = async () => {
    setTraeRefreshing(true);
    try {
      const id = await api.getTraeMachineId();
      setTraeMachineId(id);
    } catch (err: any) {
      console.error("Failed to get Trae IDE machine ID:", err);
      setTraeMachineId("Not found");
    } finally {
      setTraeRefreshing(false);
    }
  };

  // Load Trae IDE path
  const loadTraePath = async () => {
    setTraePathLoading(true);
    try {
      const path = await api.getTraePath();
      setTraePath(path);
    } catch (err: any) {
      console.error("Failed to get Trae IDE path:", err);
      setTraePath("");
    } finally {
      setTraePathLoading(false);
    }
  };

  useEffect(() => {
    loadTraeMachineId();
    loadTraePath();
  }, []);

  // Copy Trae IDE machine ID
  const handleCopyTraeMachineId = async () => {
    try {
      await navigator.clipboard.writeText(traeMachineId);
      onToast?.("success", "Trae IDE machine ID copied to clipboard");
    } catch {
      onToast?.("error", "Copy failed");
    }
  };

  // Clear Trae IDE login state
  const handleClearTraeLoginState = async () => {
    if (!confirm("Are you sure you want to clear Trae IDE login state?\n\nThis will:\n• Reset Trae IDE machine ID\n• Clear all login info\n• Delete local cache data\n\nAfter this, Trae IDE will be in a fresh install state and need to re-login.\n\nPlease make sure Trae IDE is closed!")) {
      return;
    }

    setClearingTrae(true);
    try {
      await api.clearTraeLoginState();
      await loadTraeMachineId(); // Reload new machine ID
      onToast?.("success", "Trae IDE login state cleared, please reopen Trae IDE to login");
    } catch (err: any) {
      onToast?.("error", err.message || "Clear failed");
    } finally {
      setClearingTrae(false);
    }
  };

  // Auto-scan Trae IDE path
  const handleScanTraePath = async () => {
    setScanning(true);
    try {
      const path = await api.scanTraePath();
      setTraePath(path);
      onToast?.("success", "Found Trae IDE: " + path);
    } catch (err: any) {
      onToast?.("error", err.message || "Trae IDE not found, please set path manually");
    } finally {
      setScanning(false);
    }
  };

  // Manually set Trae IDE path
  const handleSetTraePath = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: "Trae IDE",
          extensions: ["exe"]
        }],
        title: "Select Trae.exe file"
      });

      if (selected) {
        const path = selected as string;
        await api.setTraePath(path);
        setTraePath(path);
        onToast?.("success", "Trae IDE path saved");
      }
    } catch (err: any) {
      onToast?.("error", err.message || "Failed to select file");
    }
  };

  // Force re-authentication
  const handleForceReauth = async () => {
    if (!confirm("Are you sure you want to force re-authentication?\n\nThis will:\n• Clear current login state\n• Clear all pending requests\n• Require re-entering account password\n\nPlease confirm to continue?")) {
      return;
    }

    setForceReauthLoading(true);
    try {
      await forceReauth();
      onToast?.("success", "Login state cleared, please re-login");
    } catch (err: any) {
      onToast?.("error", err.message || "Force re-authentication failed");
    } finally {
      setForceReauthLoading(false);
    }
  };

  return (
    <div className="settings-page">
      {/* Machine ID */}
      <div className="settings-section">
        <h3>Machine ID</h3>
        <div className="machine-id-card trae-card">
          <div className="machine-id-header">
            <div className="machine-id-icon trae-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 2L2 7l10 5 10-5-10-5z"/>
                <path d="M2 17l10 5 10-5"/>
                <path d="M2 12l10 5 10-5"/>
              </svg>
            </div>
            <div className="machine-id-title">
              <span>MachineId</span>
              <span className="machine-id-subtitle">Unique client identifier</span>
            </div>
          </div>
          <div className="machine-id-value">
            <code>{traeRefreshing ? "Loading..." : traeMachineId}</code>
          </div>
          <div className="machine-id-actions">
            <button
              className="machine-id-btn"
              onClick={loadTraeMachineId}
              disabled={traeRefreshing}
              title="Refresh"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
              </svg>
              Refresh
            </button>
            <button
              className="machine-id-btn"
              onClick={handleCopyTraeMachineId}
              disabled={!traeMachineId || traeRefreshing || traeMachineId === "Not found"}
              title="Copy"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
              </svg>
              Copy
            </button>
            <button
              className="machine-id-btn danger"
              onClick={handleClearTraeLoginState}
              disabled={clearingTrae || traeRefreshing}
              title="Clear login state"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                <line x1="10" y1="11" x2="10" y2="17"/>
                <line x1="14" y1="11" x2="14" y2="17"/>
              </svg>
              {clearingTrae ? "Clearing..." : "Clear Login State"}
            </button>
          </div>
          <div className="machine-id-tip warning">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
              <line x1="12" y1="9" x2="12" y2="13"/>
              <line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
            <span>Clearing login state will reset machine ID and delete all login info. Client will need to re-login. Please close client first.</span>
          </div>
        </div>
      </div>

      {/* Path Settings */}
      <div className="settings-section">
        <h3>Client Path</h3>
        <div className="machine-id-card trae-card">
          <div className="machine-id-header">
            <div className="machine-id-icon trae-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
            </div>
            <div className="machine-id-title">
              <span>Installation Path</span>
              <span className="machine-id-subtitle">Used to auto-open client</span>
            </div>
          </div>
          <div className="machine-id-value">
            <code>{traePathLoading ? "Loading..." : (traePath || "Not set")}</code>
          </div>
          <div className="machine-id-actions">
            <button
              className="machine-id-btn"
              onClick={handleScanTraePath}
              disabled={scanning}
              title="Auto-scan"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="M21 21l-4.35-4.35"/>
              </svg>
              {scanning ? "Scanning..." : "Auto-Scan"}
            </button>
            <button
              className="machine-id-btn"
              onClick={handleSetTraePath}
              title="Manual set"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
              Manual Set
            </button>
          </div>
          <div className="machine-id-tip">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 16v-4"/>
              <path d="M12 8h.01"/>
            </svg>
            <span>Client will auto-open after account switch. If auto-scan can't find it, manually set the full path to Trae.exe.</span>
          </div>
        </div>
      </div>

      {/* Authentication Management */}
      <div className="settings-section">
        <h3>Authentication Management</h3>
        <div className="machine-id-card trae-card">
          <div className="machine-id-header">
            <div className="machine-id-icon trae-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
              </svg>
            </div>
            <div className="machine-id-title">
              <span>Session Management</span>
              <span className="machine-id-subtitle">Manage login state and auth info</span>
            </div>
          </div>
          <div className="machine-id-actions">
            <button
              className="machine-id-btn danger"
              onClick={handleForceReauth}
              disabled={forceReauthLoading}
              title="Force re-authentication"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
                <polyline points="16 17 21 12 16 7"/>
                <line x1="21" y1="12" x2="9" y2="12"/>
              </svg>
              {forceReauthLoading ? "Processing..." : "Force Re-Authentication"}
            </button>
          </div>
          <div className="machine-id-tip warning">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
              <line x1="12" y1="9" x2="12" y2="13"/>
              <line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
            <span>Force re-authentication will clear current auth state and all pending requests. You'll need to re-enter account password.</span>
          </div>
        </div>
      </div>

      <div className="settings-section">
        <h3>General Settings</h3>
        <div className="setting-item">
          <div className="setting-info">
            <div className="setting-label">Auto Refresh</div>
            <div className="setting-desc">Periodically auto-refresh account usage data</div>
          </div>
          <label className="toggle">
            <input type="checkbox" />
            <span className="toggle-slider"></span>
          </label>
        </div>

        <div className="setting-item">
          <div className="setting-info">
            <div className="setting-label">Refresh Interval</div>
            <div className="setting-desc">Time interval for auto-refresh (minutes)</div>
          </div>
          <select className="setting-select">
            <option value="5">5 minutes</option>
            <option value="10">10 minutes</option>
            <option value="30">30 minutes</option>
            <option value="60">60 minutes</option>
          </select>
        </div>
      </div>

      <div className="settings-section">
        <h3>Data Management</h3>
        <div className="setting-item">
          <div className="setting-info">
            <div className="setting-label">Export Data</div>
            <div className="setting-desc">Export all account data as JSON file</div>
          </div>
          <button className="setting-btn">Export</button>
        </div>

        <div className="setting-item">
          <div className="setting-info">
            <div className="setting-label">Import Data</div>
            <div className="setting-desc">Import account data from JSON file</div>
          </div>
          <button className="setting-btn">Import</button>
        </div>

        <div className="setting-item danger">
          <div className="setting-info">
            <div className="setting-label">Clear Data</div>
            <div className="setting-desc">Delete all account data (cannot be recovered)</div>
          </div>
          <button className="setting-btn danger">Clear</button>
        </div>
      </div>
    </div>
  );
}
