import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { Sidebar } from "./components/Sidebar";
import { AccountCard } from "./components/AccountCard";
import { AccountListItem } from "./components/AccountListItem";
import { AccountList, AccountListErrorBoundary } from "./components/AccountList";
import { AddAccountModal } from "./components/AddAccountModal";
import { ContextMenu } from "./components/ContextMenu";
import { DetailModal } from "./components/DetailModal";
import { Toast } from "./components/Toast";
import { ConfirmModal } from "./components/ConfirmModal";
import { InfoModal } from "./components/InfoModal";
import { UpdateTokenModal } from "./components/UpdateTokenModal";
import { ReauthDialog } from "./components/ReauthDialog";
import { SwitchProgress, SwitchProgressData } from "./components/SwitchProgress";
import { SwitchErrorModal } from "./components/SwitchErrorModal";
import { Dashboard } from "./pages/Dashboard";
import { Settings } from "./pages/Settings";
import { useToast } from "./hooks/useToast";
import { useAuth } from "./contexts/AuthContext";
import { getWebSocketClient } from "./services/websocketClient";
import * as api from "./api";
import type { AccountBrief, UsageSummary } from "./types";
import "./App.css";

interface AccountWithUsage extends AccountBrief {
  usage?: UsageSummary | null;
}

type ViewMode = "grid" | "list";

function App() {
  const [accounts, setAccounts] = useState<AccountWithUsage[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [showAddModal, setShowAddModal] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState("dashboard");
  const [viewMode, setViewMode] = useState<ViewMode>("grid");
  const useOptimizedList = true; // Use optimized list version

  // Use custom Toast hook
  const { toasts, addToast, removeToast } = useToast();
  
  // Use Auth Context
  const { showReauthDialog, setShowReauthDialog, triggerRequestRetry } = useAuth();
  
  // Export format dialog state
  const [showExportFormatDialog, setShowExportFormatDialog] = useState(false);

  // Create usage map for AccountList
  const usageMap = useMemo(() => {
    const map = new Map<string, UsageSummary>();
    accounts.forEach((account) => {
      if (account.usage) {
        map.set(account.id, account.usage);
      }
    });
    return map;
  }, [accounts]);

  // Confirmation dialog state
  const [confirmModal, setConfirmModal] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    type: "danger" | "warning" | "info";
    onConfirm: () => void;
  } | null>(null);

  // Context menu state
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    accountId: string;
  } | null>(null);

  // Detail dialog state
  const [detailAccount, setDetailAccount] = useState<AccountWithUsage | null>(null);

  // Refreshing account IDs
  const [refreshingIds, setRefreshingIds] = useState<Set<string>>(new Set());

  // Update Token dialog state
  const [updateTokenModal, setUpdateTokenModal] = useState<{
    accountId: string;
    accountName: string;
  } | null>(null);

  // Info display dialog state
  const [infoModal, setInfoModal] = useState<{
    isOpen: boolean;
    title: string;
    icon: string;
    sections: Array<{
      title?: string;
      content: string;
      type?: "text" | "code" | "list";
    }>;
    confirmText: string;
    onConfirm: () => void;
  } | null>(null);

  // Switch progress state
  const [switchProgress, setSwitchProgress] = useState<SwitchProgressData | null>(null);
  
  // Switch error state
  const [switchError, setSwitchError] = useState<{
    isOpen: boolean;
    message: string;
    accountId: string;
  }>({
    isOpen: false,
    message: "",
    accountId: "",
  });

  // WebSocket connection state
  const [wsConnected, setWsConnected] = useState(false);
  const wsClientRef = useRef(getWebSocketClient());
  const lastApiCallRef = useRef<number>(Date.now());

  // Loading progress state
  const [loadingProgress, setLoadingProgress] = useState<{
    current: number;
    total: number;
    message: string;
  } | null>(null);

  // Load account list (display list first, then load usage in background)
  const loadAccounts = useCallback(async () => {
    setLoading(true);
    setLoadingProgress({ current: 0, total: 1, message: "Loading account list..." });
    
    try {
      const list = await api.getAccounts();

      // Display account list immediately (don't wait for usage loading)
      setAccounts(list.map((account) => ({ ...account, usage: undefined })));
      setLoading(false);

      // Load usage in background with progress (with concurrency control)
      if (list.length > 0) {
        setLoadingProgress({ current: 0, total: list.length, message: "Loading usage data..." });
        
        // Use concurrency control, process max 10 requests at a time
        const concurrencyLimit = 10;
        const results: Array<{ index: number; usage: any }> = [];
        let completed = 0;

        for (let i = 0; i < list.length; i += concurrencyLimit) {
          const batch = list.slice(i, Math.min(i + concurrencyLimit, list.length));
          const batchResults = await Promise.allSettled(
            batch.map((account, batchIndex) => 
              api.getAccountUsage(account.id).then(usage => ({
                index: i + batchIndex,
                usage
              }))
            )
          );

          batchResults.forEach((result) => {
            if (result.status === 'fulfilled') {
              results.push(result.value);
            }
            completed++;
            setLoadingProgress({ 
              current: completed, 
              total: list.length, 
              message: `Loading usage data (${completed}/${list.length})...` 
            });
          });

          // Update completed accounts in real-time
          setAccounts((prev) => {
            const updated = [...prev];
            results.forEach(({ index, usage }) => {
              if (updated[index]) {
                updated[index] = { ...updated[index], usage };
              }
            });
            return updated;
          });
        }

        setLoadingProgress(null);
      } else {
        setLoadingProgress(null);
      }
    } catch (err: any) {
      setError(err.message || "Failed to load accounts");
      setLoading(false);
      setLoadingProgress(null);
    }
  }, []);

  // Initial load
  useEffect(() => {
    loadAccounts();
  }, [loadAccounts]);

  // WebSocket connection management and session change handling
  useEffect(() => {
    const wsClient = wsClientRef.current;

    console.log('[App] Initializing WebSocket connection for session change handling');

    // Listen for session_changed event
    const unsubscribeSessionChanged = wsClient.onSessionChanged((event) => {
      console.log('[App] Received session_changed event:', event);
      
      // Show notification
      addToast("info", `Account switched to ${event.email}`, 3000);
      
      // Only update is_current flag, don't reload usage data
      api.getAccounts().then((accountsList) => {
        console.log('[App] Updated account list after session change');
        setAccounts((prev) =>
          prev.map((a) => {
            const updated = accountsList.find((acc) => acc.id === a.id);
            return updated ? { ...a, is_current: updated.is_current } : a;
          })
        );
      }).catch((error) => {
        console.error('[App] Failed to update accounts after session change:', error);
        addToast("error", "Failed to update account info");
      });
    });

    // Listen for connection status changes
    const unsubscribeConnectionStatus = wsClient.onConnectionStatus((connected) => {
      console.log('[App] WebSocket connection status changed:', connected);
      setWsConnected(connected);
      
      if (connected) {
        console.log('[App] WebSocket connected successfully');
      } else {
        console.log('[App] WebSocket disconnected, will use fallback detection');
      }
    });

    // Listen for errors
    const unsubscribeError = wsClient.onError((error) => {
      console.error('[App] WebSocket error:', error);
      // Don't show error notification, will auto-reconnect
    });

    // Establish connection
    wsClient.connect();

    // Cleanup function
    return () => {
      console.log('[App] Cleaning up WebSocket subscriptions');
      unsubscribeSessionChanged();
      unsubscribeConnectionStatus();
      unsubscribeError();
      // Don't disconnect, keep connection alive during app lifecycle
    };
  }, [loadAccounts, addToast]);

  // Fallback detection: detect session changes via API when not connected
  useEffect(() => {
    // If WebSocket is connected, no need for fallback detection
    if (wsConnected) {
      return;
    }

    console.log('[App] WebSocket not connected, using fallback detection');

    // Check periodically (every 30 seconds)
    const intervalId = setInterval(async () => {
      const now = Date.now();
      const timeSinceLastCall = now - lastApiCallRef.current;
      
      // If there was a recent API call (within 30 seconds), skip check
      if (timeSinceLastCall < 30000) {
        return;
      }

      console.log('[App] Performing fallback session detection');
      
      try {
        // Detect session changes via API call
        const currentAccounts = await api.getAccounts();
        
        // Compare current active account
        const currentActiveAccount = currentAccounts.find(acc => acc.is_current);
        const previousActiveAccount = accounts.find(acc => acc.is_current);
        
        if (currentActiveAccount && previousActiveAccount && 
            currentActiveAccount.id !== previousActiveAccount.id) {
          console.log('[App] Session change detected via fallback:', {
            previous: previousActiveAccount.id,
            current: currentActiveAccount.id
          });
          
          // Only update is_current flag, don't reload usage data
          setAccounts((prev) =>
            prev.map((a) => {
              const updated = currentAccounts.find((acc) => acc.id === a.id);
              return updated ? { ...a, is_current: updated.is_current } : a;
            })
          );
          addToast("info", `Account switched to ${currentActiveAccount.email || currentActiveAccount.name}`, 3000);
        }
      } catch (error) {
        console.error('[App] Fallback session detection failed:', error);
      }
    }, 30000); // Check every 30 seconds

    return () => {
      clearInterval(intervalId);
    };
  }, [wsConnected, accounts, loadAccounts, addToast]);

  // Track API call time (for fallback detection optimization)
  useEffect(() => {
    // Record time each time accounts updates
    lastApiCallRef.current = Date.now();
  }, [accounts]);

  // Listen for backend switch events
  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenSuccess: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;

    // Listen for progress event
    listen<SwitchProgressData>("switch-progress", (event) => {
      console.log("[App] Switch progress:", event.payload);
      setSwitchProgress(event.payload);
    }).then((unlisten) => {
      unlistenProgress = unlisten;
    });

    // Listen for success event
    listen<string>("switch-success", (event) => {
      console.log("[App] Switch success:", event.payload);
      setSwitchProgress(null);
      addToast("success", event.payload, 2000);
      // Only update is_current flag, don't reload usage data
      api.getAccounts().then((accountsList) => {
        setAccounts((prev) =>
          prev.map((a) => {
            const updated = accountsList.find((acc) => acc.id === a.id);
            return updated ? { ...a, is_current: updated.is_current } : a;
          })
        );
      }).catch((error) => {
        console.error('[App] Failed to update accounts after switch success:', error);
      });
    }).then((unlisten) => {
      unlistenSuccess = unlisten;
    });

    // Listen for error event
    listen<string>("switch-error", (event) => {
      console.log("[App] Switch error:", event.payload);
      setSwitchProgress(null);
      // Extract account ID from error message if available
      const accountId = ""; // Get from context if needed
      setSwitchError({
        isOpen: true,
        message: event.payload,
        accountId,
      });
    }).then((unlisten) => {
      unlistenError = unlisten;
    });

    // Cleanup listeners
    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenSuccess) unlistenSuccess();
      if (unlistenError) unlistenError();
    };
  }, [addToast, loadAccounts]);

  // Auto-refresh tokens about to expire
  useEffect(() => {
    // Refresh on startup
    api.refreshAllTokens().then((refreshed) => {
      if (refreshed.length > 0) {
        console.log(`[INFO] Auto-refreshed ${refreshed.length} tokens on startup`);
        loadAccounts();
      }
    }).catch(console.error);

    // Refresh every 30 minutes
    const interval = setInterval(() => {
      api.refreshAllTokens().then((refreshed) => {
        if (refreshed.length > 0) {
          console.log(`[INFO] Auto-refreshed ${refreshed.length} tokens on schedule`);
          loadAccounts();
        }
      }).catch(console.error);
    }, 30 * 60 * 1000);

    return () => clearInterval(interval);
  }, [loadAccounts]);

  // Add account
  const handleAddAccount = async (token: string, cookies?: string) => {
    await api.addAccountByToken(token, cookies);
    addToast("success", "Account added successfully");
    await loadAccounts();
  };

  // Delete account
  const handleDeleteAccount = async (accountId: string) => {
    setConfirmModal({
      isOpen: true,
      title: "Delete Account",
      message: "Are you sure you want to delete this account? This action cannot be undone.",
      type: "danger",
      onConfirm: async () => {
        try {
          await api.removeAccount(accountId);
          setSelectedIds((prev) => {
            const next = new Set(prev);
            next.delete(accountId);
            return next;
          });
          addToast("success", "Account deleted");
          await loadAccounts();
        } catch (err: any) {
          addToast("error", err.message || "Failed to delete account");
        }
        setConfirmModal(null);
      },
    });
  };

  // Refresh single account
  const handleRefreshAccount = async (accountId: string) => {
    // Prevent duplicate refresh
    if (refreshingIds.has(accountId)) {
      return;
    }

    setRefreshingIds((prev) => new Set(prev).add(accountId));

    try {
      const usage = await api.getAccountUsage(accountId);
      setAccounts((prev) =>
        prev.map((a) => (a.id === accountId ? { ...a, usage } : a))
      );
      addToast("success", "Data refreshed successfully");
    } catch (err: any) {
      addToast("error", err.message || "Refresh failed");
    } finally {
      setRefreshingIds((prev) => {
        const next = new Set(prev);
        next.delete(accountId);
        return next;
      });
    }
  };

  // Select account
  const handleSelectAccount = (accountId: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(accountId)) {
        next.delete(accountId);
      } else {
        next.add(accountId);
      }
      return next;
    });
  };

  // Select all / Deselect all
  const handleSelectAll = () => {
    if (selectedIds.size === accounts.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(accounts.map((a) => a.id)));
    }
  };

  // Context menu
  const handleContextMenu = (e: React.MouseEvent, accountId: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, accountId });
  };

  // Re-login account
  const handleReloginAccount = async (accountId: string) => {
    try {
      addToast("info", "Re-logging in...");
      await api.reloginAccount(accountId);
      addToast("success", "Re-login successful, Trae IDE refreshed");
    } catch (err: any) {
      addToast("error", err.message || "Re-login failed");
    }
  };

  // Copy Token
  const handleCopyToken = async (accountId: string) => {
    try {
      const account = await api.getAccount(accountId);
      if (account.jwt_token) {
        await navigator.clipboard.writeText(account.jwt_token);
        addToast("success", "Token copied to clipboard");
      } else {
        addToast("warning", "This account has no valid Token");
      }
    } catch (err: any) {
      addToast("error", err.message || "Failed to get Token");
    }
  };

  // Switch account
  const handleSwitchAccount = async (accountId: string) => {
    const account = accounts.find((a) => a.id === accountId);
    if (!account) return;

    setConfirmModal({
      isOpen: true,
      title: "Switch Account",
      message: `Are you sure you want to switch to account "${account.email || account.name}"?\n\nThe system will automatically close Trae IDE and switch login info.`,
      type: "warning",
      onConfirm: async () => {
        setConfirmModal(null);
        addToast("info", "Switching account, please wait...");
        try {
          await api.switchAccount(accountId);
          
          // Only update is_current flag, don't reload usage
          setAccounts((prev) =>
            prev.map((a) => ({
              ...a,
              is_current: a.id === accountId,
            }))
          );
          
          addToast("success", "Account switched successfully, please reopen Trae IDE");
        } catch (err: any) {
          addToast("error", err.message || "Failed to switch account");
        }
      },
    });
  };

  // Retry account switch
  const handleRetrySwitchAccount = async (accountId: string) => {
    console.log("[App] Retrying account switch:", accountId);
    setSwitchError({ isOpen: false, message: "", accountId: "" });
    try {
      await api.switchAccount(accountId);
      // Only update is_current flag, don't reload usage data
      const accountsList = await api.getAccounts();
      setAccounts((prev) =>
        prev.map((a) => {
          const updated = accountsList.find((acc) => acc.id === a.id);
          return updated ? { ...a, is_current: updated.is_current } : a;
        })
      );
    } catch (err: any) {
      setSwitchError({
        isOpen: true,
        message: err.message || "Failed to switch account",
        accountId,
      });
    }
  };

  // Rollback to previous account
  const handleRollbackAccount = async () => {
    console.log("[App] Rolling back to previous account");
    setSwitchError({ isOpen: false, message: "", accountId: "" });
    try {
      addToast("info", "Rolling back to previous account...");
      await api.rollbackAccount();
      
      // Only update is_current flag, don't reload usage
      const accounts_list = await api.getAccounts();
      setAccounts((prev) =>
        prev.map((a) => {
          const updated = accounts_list.find((acc) => acc.id === a.id);
          return updated ? { ...a, is_current: updated.is_current } : a;
        })
      );
      
      addToast("success", "Rolled back to previous account");
    } catch (err: any) {
      addToast("error", err.message || "Rollback failed");
    }
  };

  // View details
  const handleViewDetail = async (accountId: string) => {
    const account = accounts.find((a) => a.id === accountId);
    if (account) {
      try {
        // Get full account info (including token and cookies)
        const fullAccount = await api.getAccount(accountId);
        setDetailAccount({ ...account, ...fullAccount });
      } catch (err: any) {
        addToast("error", "Failed to get account details");
        console.error("Failed to get account details:", err);
      }
    }
  };

  // Update Token
  const handleUpdateToken = async (accountId: string, token: string) => {
    try {
      const usage = await api.updateAccountToken(accountId, token);
      setAccounts((prev) =>
        prev.map((a) => (a.id === accountId ? { ...a, usage } : a))
      );
      addToast("success", "Token updated successfully, data refreshed");
    } catch (err: any) {
      throw err; // Let dialog show error
    }
  };

  // Open update Token dialog
  const handleOpenUpdateToken = (accountId: string) => {
    const account = accounts.find((a) => a.id === accountId);
    if (account) {
      setUpdateTokenModal({
        accountId,
        accountName: account.email || account.name,
      });
    }
  };

  // Claim gift
  const handleClaimGift = async (accountId: string) => {
    const account = accounts.find((a) => a.id === accountId);
    if (!account) return;

    setConfirmModal({
      isOpen: true,
      title: "Claim Gift",
      message: `Are you sure you want to claim the anniversary gift for account "${account.email || account.name}"?\n\nAccount quota will be automatically refreshed after claiming.`,
      type: "info",
      onConfirm: async () => {
        setConfirmModal(null);
        addToast("info", "Claiming gift, please wait...");
        try {
          await api.claimGift(accountId);
          // Refresh account data
          await handleRefreshAccount(accountId);
          addToast("success", "Gift claimed successfully! Quota updated");
        } catch (err: any) {
          addToast("error", err.message || "Failed to claim gift");
        }
      },
    });
  };

  // Show export info
  const handleShowExportInfo = () => {
    if (accounts.length === 0) {
      addToast("warning", "No accounts to export");
      return;
    }

    setInfoModal({
      isOpen: true,
      title: "Export Account Instructions",
      icon: "📤",
      sections: [
        {
          title: "📄 Export Format",
          content: "JSON file (.json)",
          type: "text"
        },
        {
          title: "📁 Save Location",
          content: "Browser default download folder\nFile name format: trae-accounts-YYYY-MM-DD.json",
          type: "text"
        },
        {
          title: "📋 File Content",
          content: `<ul>
<li>Complete info for all accounts</li>
<li>Token and Cookies data</li>
<li>Usage statistics</li>
<li>Account creation and update time</li>
</ul>`,
          type: "list"
        },
        {
          title: "✅ After Export You Can",
          content: `<ul>
<li>Backup account data</li>
<li>Migrate to other devices</li>
<li>Recover accidentally deleted accounts</li>
<li>Share with other devices</li>
</ul>`,
          type: "list"
        },
        {
          title: "⚠️ Security Notice",
          content: `<ul>
<li><strong>Export file contains sensitive information</strong></li>
<li><strong>Keep exported file safe</strong></li>
<li><strong>Do not share with others</strong></li>
<li>Recommend encrypting exported file</li>
</ul>`,
          type: "list"
        },
        {
          content: `Will export ${accounts.length} accounts`,
          type: "text"
        }
      ],
      confirmText: "Start Export",
      onConfirm: () => {
        setInfoModal(null);
        handleExportAccounts();
      }
    });
  };

  // Export accounts
  const handleExportAccounts = async () => {
    try {
      const data = await api.exportAccounts();
      const blob = new Blob([data], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      const fileName = `trae-accounts-${new Date().toISOString().split("T")[0]}.json`;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      addToast("success", `Exported ${accounts.length} accounts to download folder: ${fileName}`);
    } catch (err: any) {
      addToast("error", err.message || "Export failed");
    }
  };

  // Show import info
  const handleShowImportInfo = () => {
    setInfoModal({
      isOpen: true,
      title: "Import Account Instructions",
      icon: "📥",
      sections: [
        {
          title: "📄 File Format",
          content: "JSON file (.json)",
          type: "text"
        },
        {
          title: "📋 File Structure Example",
          content: `{
  "accounts": [
    {
      "id": "account_id",
      "name": "username",
      "email": "email_address",
      "jwt_token": "token_string",
      "cookies": "cookies_string",
      "plan_type": "plan_type",
      "created_at": timestamp,
      "is_active": true,
      ...
    }
  ],
  "active_account_id": "current_active_account_id",
  "current_account_id": "current_using_account_id"
}`,
          type: "code"
        },
        {
          title: "✅ Import Steps",
          content: `<ul>
<li>After confirming, select JSON file</li>
<li>System automatically validates format</li>
<li>Import all valid accounts</li>
</ul>`,
          type: "list"
        },
        {
          title: "⚠️ Notes",
          content: `<ul>
<li>Only supports format exported by this app</li>
<li>Import automatically skips duplicate accounts</li>
<li>Recommend regular backup of account data</li>
</ul>`,
          type: "list"
        }
      ],
      confirmText: "Select File",
      onConfirm: () => {
        setInfoModal(null);
        handleImportAccounts();
      }
    });
  };

  // Import state
  const [importing, setImporting] = useState(false);

  // Import accounts
  const handleImportAccounts = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      setImporting(true);
      
      try {
        // Read file
        addToast("info", "Reading file...");
        const text = await file.text();
        
        // Parse JSON to get account count
        let accountCount = 0;
        try {
          const data = JSON.parse(text);
          accountCount = Array.isArray(data) ? data.length : 0;
        } catch {
          accountCount = 0;
        }
        
        // Show importing hint
        if (accountCount > 0) {
          addToast("info", `Importing ${accountCount} accounts, please wait...`, 0);
        } else {
          addToast("info", "Importing accounts, please wait...", 0);
        }
        
        // Call import API
        const count = await api.importAccounts(text);
        
        // Import complete, close dialog immediately
        setImporting(false);
        
        // Import successful, show success hint
        addToast("success", `Successfully imported ${count} accounts`);
        
        // Reload account list (will show progress bar)
        await loadAccounts();
      } catch (err: any) {
        addToast("error", err.message || "Import failed");
        setImporting(false);
      }
    };
    input.click();
  };

  // Batch refresh selected accounts (optimized: parallel processing with progress feedback)
  const handleBatchRefresh = async () => {
    if (selectedIds.size === 0) {
      addToast("warning", "Please select accounts to refresh first");
      return;
    }

    const ids = Array.from(selectedIds);
    addToast("info", `Refreshing ${ids.length} accounts...`);

    // Refresh all selected accounts in parallel
    const results = await Promise.allSettled(
      ids.map(async (id) => {
        try {
          const usage = await api.getAccountUsage(id);
          setAccounts((prev) =>
            prev.map((a) => (a.id === id ? { ...a, usage } : a))
          );
          return { id, success: true };
        } catch (err: any) {
          return { id, success: false, error: err.message };
        }
      })
    );

    // Count results
    const successCount = results.filter(
      (r) => r.status === 'fulfilled' && r.value.success
    ).length;
    const failCount = ids.length - successCount;

    if (failCount === 0) {
      addToast("success", `Successfully refreshed ${successCount} accounts`);
    } else {
      addToast("warning", `Refresh complete: ${successCount} succeeded, ${failCount} failed`);
    }
  };

  // Batch delete selected accounts (optimized: improved error handling and feedback)
  const handleBatchDelete = () => {
    if (selectedIds.size === 0) {
      addToast("warning", "Please select accounts to delete first");
      return;
    }

    const ids = Array.from(selectedIds);
    setConfirmModal({
      isOpen: true,
      title: "Batch Delete",
      message: `Are you sure you want to delete ${ids.length} selected accounts? This action cannot be undone.`,
      type: "danger",
      onConfirm: async () => {
        setConfirmModal(null);
        addToast("info", `Deleting ${ids.length} accounts...`);

        // Delete all selected accounts in parallel
        const results = await Promise.allSettled(
          ids.map((id) => api.removeAccount(id))
        );

        // Count results
        const successCount = results.filter((r) => r.status === 'fulfilled').length;
        const failCount = ids.length - successCount;

        setSelectedIds(new Set());
        await loadAccounts();

        if (failCount === 0) {
          addToast("success", `Successfully deleted ${successCount} accounts`);
        } else {
          addToast("warning", `Delete complete: ${successCount} succeeded, ${failCount} failed`);
        }
      },
    });
  };

  // Delete expired/invalid accounts
  const handleDeleteExpiredAccounts = () => {
    // Filter expired or invalid accounts
    const expiredAccounts = accounts.filter((account) => {
      if (!account.token_expired_at) return false;
      const expiry = new Date(account.token_expired_at).getTime();
      if (isNaN(expiry)) return false;
      return expiry < Date.now(); // Token expired
    });

    if (expiredAccounts.length === 0) {
      addToast("info", "No expired or invalid accounts found");
      return;
    }

    setConfirmModal({
      isOpen: true,
      title: "Delete Expired Accounts",
      message: `Detected ${expiredAccounts.length} expired accounts. Are you sure you want to delete them? This action cannot be undone.`,
      type: "warning",
      onConfirm: async () => {
        setConfirmModal(null);
        addToast("info", `Deleting ${expiredAccounts.length} expired accounts...`);

        // Delete all expired accounts in parallel
        const results = await Promise.allSettled(
          expiredAccounts.map((account) => api.removeAccount(account.id))
        );

        // Count results
        const successCount = results.filter((r) => r.status === 'fulfilled').length;
        const failCount = expiredAccounts.length - successCount;

        setSelectedIds(new Set());
        await loadAccounts();

        if (failCount === 0) {
          addToast("success", `Successfully deleted ${successCount} expired accounts`);
        } else {
          addToast("warning", `Delete complete: ${successCount} succeeded, ${failCount} failed`);
        }
      },
    });
  };
  
  // Selective export accounts
  const handleExportSelected = () => {
    if (selectedIds.size === 0) {
      addToast("warning", "Please select accounts to export first");
      return;
    }
    
    // Show format selection dialog
    setShowExportFormatDialog(true);
  };
  
  // Execute export
  const handleConfirmExport = async (format: 'csv' | 'json') => {
    setShowExportFormatDialog(false);
    
    const accountIds = Array.from(selectedIds);
    
    try {
      addToast("info", `Preparing to export ${accountIds.length} accounts...`);
      
      // Get full data for accounts to export
      const accountsToExport = await Promise.all(
        accountIds.map(id => api.getAccount(id))
      );
      
      // Use Tauri save dialog
      const { save } = await import('@tauri-apps/plugin-dialog');
      
      const timestamp = new Date().toISOString().split('T')[0];
      const defaultFileName = `trae-accounts-selected-${timestamp}.${format}`;
      
      // Show save dialog
      const filePath = await save({
        defaultPath: defaultFileName,
        filters: [{
          name: format === 'json' ? 'JSON File' : 'CSV File',
          extensions: [format]
        }]
      });
      
      // User cancelled save
      if (!filePath) {
        addToast("info", "Export cancelled");
        return;
      }
      
      addToast("info", `Exporting to ${filePath}...`);
      
      // Prepare export data
      let content: string;
      if (format === 'json') {
        content = JSON.stringify(accountsToExport, null, 2);
      } else {
        // CSV format
        const headers = ['ID', 'Name', 'Email', 'Plan Type', 'Token', 'Cookies', 'Created At'];
        const rows = accountsToExport.map(acc => [
          acc.id,
          acc.name,
          acc.email || '',
          acc.plan_type || '',
          acc.jwt_token || '',
          acc.cookies || '',
          new Date(acc.created_at).toISOString()
        ]);
        content = [headers, ...rows].map(row => row.join(',')).join('\n');
      }
      
      // Use Tauri file system API to write file
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      await writeTextFile(filePath, content);
      
      addToast("success", `Successfully exported ${accountIds.length} accounts`);
    } catch (error: any) {
      addToast("error", error.message || "Export failed");
    }
  };

  /**
   * Handle successful re-authentication.
   * 
   * When user successfully logs in in Reauth Dialog, close dialog and reload account list.
   */
  const handleReauthSuccess = useCallback(async () => {
    console.log('[App] Reauth successful, closing dialog');
    setShowReauthDialog(false);
    addToast("success", "Re-login successful");
    
    try {
      // Trigger request retry
      await triggerRequestRetry();
      console.log('[App] Request retry completed after reauth');
    } catch (error) {
      console.error('[App] Request retry failed after reauth:', error);
      addToast("warning", "Some request retries failed");
    }
    
    // Reload account list to ensure data sync
    loadAccounts();
  }, [setShowReauthDialog, addToast, triggerRequestRetry, loadAccounts]);

  /**
   * Handle re-authentication dialog close.
   * 
   * When user cancels re-authentication, close dialog.
   */
  const handleReauthClose = useCallback(() => {
    console.log('[App] Reauth dialog closed by user');
    setShowReauthDialog(false);
  }, [setShowReauthDialog]);

  return (
    <div className="app">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />

      <div className="app-content">
        {error && (
          <div className="error-banner">
            {error}
            <button onClick={() => setError(null)}>×</button>
          </div>
        )}

        {currentPage === "dashboard" && (
          <Dashboard accounts={accounts} />
        )}

        {currentPage === "accounts" && (
          <>
            <header className="page-header">
              <div className="header-left">
                <h2 className="page-title">Account Management</h2>
                <p>Manage your accounts</p>
              </div>
              <div className="header-right">
                <span className="account-count">Total {accounts.length} accounts</span>
                <button
                  className="header-btn danger"
                  onClick={handleDeleteExpiredAccounts}
                  title="Delete all expired accounts"
                  disabled={accounts.length === 0}
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="16" height="16">
                    <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                    <line x1="10" y1="11" x2="10" y2="17"/>
                    <line x1="14" y1="11" x2="14" y2="17"/>
                  </svg>
                  Delete Expired
                  {(() => {
                    const expiredCount = accounts.filter((account) => {
                      if (!account.token_expired_at) return false;
                      const expiry = new Date(account.token_expired_at).getTime();
                      if (isNaN(expiry)) return false;
                      return expiry < Date.now();
                    }).length;
                    return expiredCount > 0 ? <span className="badge-count">{expiredCount}</span> : null;
                  })()}
                </button>
                <button className="header-btn" onClick={handleShowImportInfo} title="Import accounts">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="16" height="16">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
                  </svg>
                  Import
                </button>
                <button className="header-btn" onClick={handleShowExportInfo} title="Export accounts" disabled={accounts.length === 0}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="16" height="16">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                  </svg>
                  Export
                </button>
                <button className="add-btn" onClick={() => setShowAddModal(true)}>
                  <span>+</span> Add Account
                </button>
              </div>
            </header>

            <main className="app-main">
              {/* 加載進度條 */}
              {loadingProgress && (
                <div className="loading-progress-bar">
                  <div className="progress-info">
                    <span>{loadingProgress.message}</span>
                    <span>{Math.round((loadingProgress.current / loadingProgress.total) * 100)}%</span>
                  </div>
                  <div className="progress-bar">
                    <div 
                      className="progress-fill" 
                      style={{ width: `${(loadingProgress.current / loadingProgress.total) * 100}%` }}
                    />
                  </div>
                </div>
              )}

              {accounts.length > 0 && (
                <div className="toolbar">
                  <div className="toolbar-left">
                    <label className="select-all">
                      <input
                        type="checkbox"
                        checked={selectedIds.size === accounts.length && accounts.length > 0}
                        onChange={handleSelectAll}
                      />
                      Select All ({selectedIds.size}/{accounts.length})
                    </label>
                    {selectedIds.size > 0 && (
                      <div className="batch-actions">
                        <button className="batch-btn" onClick={handleBatchRefresh}>
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="14" height="14">
                            <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
                          </svg>
                          Refresh
                        </button>
                        <button className="batch-btn" onClick={handleExportSelected}>
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="14" height="14">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                          </svg>
                          Export Selected
                        </button>
                        <button className="batch-btn danger" onClick={handleBatchDelete}>
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="14" height="14">
                            <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                          </svg>
                          Delete
                        </button>
                      </div>
                    )}
                  </div>
                  <div className="toolbar-right">
                    <div className="view-toggle">
                      <button
                        className={`view-btn ${viewMode === "grid" ? "active" : ""}`}
                        onClick={() => setViewMode("grid")}
                        title="Card View"
                      >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="16" height="16">
                          <rect x="3" y="3" width="7" height="7"/>
                          <rect x="14" y="3" width="7" height="7"/>
                          <rect x="3" y="14" width="7" height="7"/>
                          <rect x="14" y="14" width="7" height="7"/>
                        </svg>
                      </button>
                      <button
                        className={`view-btn ${viewMode === "list" ? "active" : ""}`}
                        onClick={() => setViewMode("list")}
                        title="List View"
                      >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="16" height="16">
                          <line x1="8" y1="6" x2="21" y2="6"/>
                          <line x1="8" y1="12" x2="21" y2="12"/>
                          <line x1="8" y1="18" x2="21" y2="18"/>
                          <line x1="3" y1="6" x2="3.01" y2="6"/>
                          <line x1="3" y1="12" x2="3.01" y2="12"/>
                          <line x1="3" y1="18" x2="3.01" y2="18"/>
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              )}

              {loading ? (
                <div className="loading">
                  <div className="spinner"></div>
                  <p>Loading...</p>
                </div>
              ) : accounts.length === 0 ? (
                <div className="empty-state">
                  <div className="empty-icon">📋</div>
                  <h3>No Accounts</h3>
                  <p>Click the button above to add an account, or import existing accounts</p>
                  <div className="empty-actions">
                    <button className="empty-btn primary" onClick={() => setShowAddModal(true)}>
                      Add Account
                    </button>
                    <button className="empty-btn" onClick={handleImportAccounts}>
                      Import Accounts
                    </button>
                  </div>
                </div>
              ) : useOptimizedList ? (
                // 使用優化版 AccountList 元件（支援虛擬滾動、分頁載入、快取）
                <AccountListErrorBoundary>
                  <AccountList
                    viewMode={viewMode === "grid" ? "card" : "list"}
                    selectedIds={selectedIds}
                    accounts={accounts}
                    onAccountSelect={handleSelectAccount}
                    onAccountContextMenu={handleContextMenu}
                    usageMap={usageMap}
                    config={{
                      pageSize: 50,
                      bufferSize: 5,
                      cacheTimeout: 5 * 60 * 1000,
                      triggerThreshold: 0.8,
                    }}
                  />
                </AccountListErrorBoundary>
              ) : viewMode === "grid" ? (
                // 傳統卡片視圖（保留作為降級選項）
                <div className="account-grid">
                  {accounts.map((account) => (
                    <AccountCard
                      key={account.id}
                      account={account}
                      usage={account.usage || null}
                      selected={selectedIds.has(account.id)}
                      onSelect={handleSelectAccount}
                      onContextMenu={handleContextMenu}
                    />
                  ))}
                </div>
              ) : (
                // 傳統列表視圖（保留作為降級選項）
                <div className="account-list">
                  <div className="list-header">
                    <div className="list-col checkbox"></div>
                    <div className="list-col avatar"></div>
                    <div className="list-col info">Account Info</div>
                    <div className="list-col plan">Plan</div>
                    <div className="list-col usage">Usage</div>
                    <div className="list-col reset">Reset Time</div>
                    <div className="list-col status">Status</div>
                    <div className="list-col actions"></div>
                  </div>
                  {accounts.map((account) => (
                    <AccountListItem
                      key={account.id}
                      account={account}
                      usage={account.usage || null}
                      selected={selectedIds.has(account.id)}
                      onSelect={handleSelectAccount}
                      onContextMenu={handleContextMenu}
                    />
                  ))}
                </div>
              )}
            </main>
          </>
        )}

        {currentPage === "settings" && (
          <>
            <header className="page-header">
              <div className="header-left">
                <h2 className="page-title">Settings</h2>
                <p>Configure application options</p>
              </div>
            </header>
            <Settings onToast={addToast} />
          </>
        )}
      </div>

      {/* Toast 通知 */}
      <Toast messages={toasts} onRemove={removeToast} />

      {/* 導入加載遮罩 */}
      {importing && (
        <div className="import-overlay">
          <div className="import-modal">
            <div className="import-spinner">
              <div className="spinner-ring"></div>
              <div className="spinner-ring"></div>
              <div className="spinner-ring"></div>
            </div>
            <h3>Importing Accounts</h3>
            <p>Please wait, processing your account data...</p>
            <div className="import-tips">
              <span>💡 Tip: Do not close the window during import</span>
            </div>
          </div>
        </div>
      )}

      {/* 确认弹窗 */}
      {confirmModal && (
        <ConfirmModal
          isOpen={confirmModal.isOpen}
          title={confirmModal.title}
          message={confirmModal.message}
          type={confirmModal.type}
          confirmText="Confirm"
          cancelText="Cancel"
          onConfirm={confirmModal.onConfirm}
          onCancel={() => setConfirmModal(null)}
        />
      )}

      {/* 信息展示弹窗 */}
      {infoModal && (
        <InfoModal
          isOpen={infoModal.isOpen}
          title={infoModal.title}
          icon={infoModal.icon}
          sections={infoModal.sections}
          confirmText={infoModal.confirmText}
          onConfirm={infoModal.onConfirm}
          onCancel={() => setInfoModal(null)}
        />
      )}

      {/* 右键菜单 */}
      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          onClose={() => setContextMenu(null)}
          onViewDetail={() => {
            handleViewDetail(contextMenu.accountId);
            setContextMenu(null);
          }}
          onRefresh={() => {
            handleRefreshAccount(contextMenu.accountId);
            setContextMenu(null);
          }}
          onRelogin={() => {
            handleReloginAccount(contextMenu.accountId);
            setContextMenu(null);
          }}
          onUpdateToken={() => {
            handleOpenUpdateToken(contextMenu.accountId);
            setContextMenu(null);
          }}
          onCopyToken={() => {
            handleCopyToken(contextMenu.accountId);
            setContextMenu(null);
          }}
          onSwitchAccount={() => {
            handleSwitchAccount(contextMenu.accountId);
            setContextMenu(null);
          }}
          onClaimGift={() => {
            handleClaimGift(contextMenu.accountId);
            setContextMenu(null);
          }}
          onDelete={() => {
            handleDeleteAccount(contextMenu.accountId);
            setContextMenu(null);
          }}
          isCurrent={accounts.find(a => a.id === contextMenu.accountId)?.is_current || false}
        />
      )}

      {/* 添加账号弹窗 */}
      <AddAccountModal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        onAdd={handleAddAccount}
        onToast={addToast}
        onAccountAdded={loadAccounts}
      />

      {/* 详情弹窗 */}
      <DetailModal
        isOpen={!!detailAccount}
        onClose={() => setDetailAccount(null)}
        account={detailAccount}
        usage={detailAccount?.usage || null}
      />

      {/* 更新 Token 弹窗 */}
      <UpdateTokenModal
        isOpen={!!updateTokenModal}
        accountId={updateTokenModal?.accountId || ""}
        accountName={updateTokenModal?.accountName || ""}
        onClose={() => setUpdateTokenModal(null)}
        onUpdate={handleUpdateToken}
      />
      
      {/* 匯出格式選擇對話框 */}
      {showExportFormatDialog && (
        <ConfirmModal
          isOpen={showExportFormatDialog}
          title="Select Export Format"
          message={`About to export ${selectedIds.size} accounts, please select export format:`}
          type="info"
          confirmText="CSV Format"
          cancelText="JSON Format"
          onConfirm={() => handleConfirmExport('csv')}
          onCancel={() => handleConfirmExport('json')}
        />
      )}

      {/* 重新登入對話框 */}
      <ReauthDialog
        open={showReauthDialog}
        onClose={handleReauthClose}
        onSuccess={handleReauthSuccess}
      />

      {/* 切換進度指示器 */}
      <SwitchProgress progress={switchProgress} />

      {/* 切換錯誤對話框 */}
      <SwitchErrorModal
        isOpen={switchError.isOpen}
        errorMessage={switchError.message}
        accountId={switchError.accountId}
        onRetry={handleRetrySwitchAccount}
        onRollback={handleRollbackAccount}
        onClose={() => setSwitchError({ isOpen: false, message: "", accountId: "" })}
      />
    </div>
  );
}

export default App;
