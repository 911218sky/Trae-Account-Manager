import { createContext, useContext, useState, useCallback, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getRequestQueue, setShowReauthDialogCallback } from '../auth';

/**
 * Authentication state interface.
 */
export interface AuthState {
  isAuthenticated: boolean;
  isRefreshing: boolean;
  showReauthDialog: boolean;
  error: string | null;
}

/**
 * Auth Context value interface.
 */
export interface AuthContextValue extends AuthState {
  refreshToken: () => Promise<void>;
  forceReauth: () => Promise<void>;
  setShowReauthDialog: (show: boolean) => void;
  clearError: () => void;
  triggerRequestRetry: () => Promise<void>;
}

/**
 * Auth Context.
 * 
 * Provides global authentication state and methods.
 * Coordinates communication between components.
 * 
 * - Manages authentication state
 * - Handles token refresh flow
 * - Triggers reauth dialog on refresh failure
 * - Provides force reauth functionality
 * - Manages request queue retry
 */
const AuthContext = createContext<AuthContextValue | undefined>(undefined);

/**
 * Auth Provider Props.
 */
interface AuthProviderProps {
  children: ReactNode;
}

/**
 * Auth Provider component.
 * 
 * Manages global authentication state and provides auth-related functionality.
 * 
 * @param props - Provider props
 * @returns Auth Provider component
 */
export function AuthProvider({ children }: AuthProviderProps) {
  const [state, setState] = useState<AuthState>({
    isAuthenticated: true, // Default to authenticated
    isRefreshing: false,
    showReauthDialog: false,
    error: null,
  });

  /**
   * Refresh token.
   * 
   * Calls backend refresh_active_token command to refresh the current active account's token.
   * 
   * - Shows loading indicator
   * - Calls backend refresh command
   * - Updates authentication state
   * - Shows reauth dialog on failure
   */
  const refreshToken = useCallback(async () => {
    try {
      // Set isRefreshing to true, show loading indicator
      setState(prev => ({ ...prev, isRefreshing: true, error: null }));

      // Call backend refresh command
      await invoke('refresh_active_token');

      // Refresh successful
      setState(prev => ({
        ...prev,
        isRefreshing: false,
        isAuthenticated: true,
        error: null,
      }));

      console.log('[AuthContext] Token refresh successful');
    } catch (error: any) {
      // Refresh failed
      console.error('[AuthContext] Token refresh failed:', error);

      // Hide loading indicator, show Reauth Dialog
      setState(prev => ({
        ...prev,
        isRefreshing: false,
        showReauthDialog: true,
        error: typeof error === 'string' ? error : error.message || 'Failed to refresh token',
      }));

      throw error;
    }
  }, []);

  /**
   * Force re-authentication.
   * 
   * Clears local tokens and triggers reauth dialog.
   * 
   * - Calls backend force_reauth command
   * - Clears local tokens
   * - Updates authentication state
   * - Shows reauth dialog
   */
  const forceReauth = useCallback(async () => {
    try {
      console.log('[AuthContext] Force reauth initiated');

      // Call backend force_reauth command
      await invoke('force_reauth');

      // Update state
      setState({
        isAuthenticated: false,
        isRefreshing: false,
        showReauthDialog: true,
        error: null,
      });

      // Log event
      console.log('[AuthContext] Force reauth completed, showing reauth dialog');
    } catch (error: any) {
      console.error('[AuthContext] Force reauth failed:', error);

      setState(prev => ({
        ...prev,
        error: typeof error === 'string' ? error : error.message || 'Failed to force reauth',
      }));

      throw error;
    }
  }, []);

  /**
   * Set reauth dialog visibility.
   * 
   * @param show - Whether to show the dialog
   */
  const setShowReauthDialog = useCallback((show: boolean) => {
    setState(prev => ({ ...prev, showReauthDialog: show }));
  }, []);

  /**
   * Register reauth dialog callback.
   * 
   * On component mount, register callback function to show reauth dialog.
   * When token refresh fails, Auth Interceptor calls this callback to show the dialog.
   */
  useEffect(() => {
    // Register callback function
    setShowReauthDialogCallback(() => {
      console.log('[AuthContext] Reauth Dialog callback triggered');
      setState(prev => ({ ...prev, showReauthDialog: true }));
    });

    console.log('[AuthContext] Reauth Dialog callback registered');
  }, []);

  /**
   * Clear error message.
   */
  const clearError = useCallback(() => {
    setState(prev => ({ ...prev, error: null }));
  }, []);

  /**
   * Trigger request retry.
   * 
   * After successful re-authentication, process all suspended requests in Request Queue.
   */
  const triggerRequestRetry = useCallback(async () => {
    try {
      console.log('[AuthContext] Triggering request retry...');
      
      const requestQueue = getRequestQueue();
      
      // Process all requests in queue
      await requestQueue.processQueue();
      
      console.log('[AuthContext] Request retry completed');
    } catch (error: any) {
      console.error('[AuthContext] Request retry failed:', error);
      throw error;
    }
  }, []);

  const value: AuthContextValue = {
    ...state,
    refreshToken,
    forceReauth,
    setShowReauthDialog,
    clearError,
    triggerRequestRetry,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

/**
 * useAuth Hook.
 * 
 * Provides access to Auth Context.
 * 
 * @returns Auth Context value
 * @throws If used outside AuthProvider
 */
export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  
  return context;
}
