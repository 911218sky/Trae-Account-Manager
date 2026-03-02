/**
 * Auth Integration Module.
 * 
 * Provides convenient way to use Auth Interceptor and Request Queue.
 * Integrates all authentication-related components with unified API.
 */

import { AuthInterceptor, InterceptorConfig, AuthError } from './interceptor';
import { RequestQueue } from './requestQueue';
import { refreshActiveToken } from '../api';

/**
 * Global Request Queue instance.
 */
let requestQueueInstance: RequestQueue | null = null;

/**
 * Global Auth Interceptor instance.
 */
let authInterceptorInstance: AuthInterceptor | null = null;

/**
 * Reauth Dialog display callback function.
 * Set by Auth Context to show dialog when refresh fails.
 */
let showReauthDialogCallback: (() => void) | null = null;

/**
 * Set Reauth Dialog display callback.
 * 
 * @param callback - Callback function to show Reauth Dialog
 */
export function setShowReauthDialogCallback(callback: () => void): void {
  showReauthDialogCallback = callback;
}

/**
 * Get or create Request Queue instance.
 * 
 * @returns Request Queue instance
 */
export function getRequestQueue(): RequestQueue {
  if (!requestQueueInstance) {
    requestQueueInstance = new RequestQueue();
  }
  return requestQueueInstance;
}

/**
 * Get or create Auth Interceptor instance.
 * 
 * @param config - Interceptor configuration (optional, only needed on first creation)
 * @returns Auth Interceptor instance
 */
export function getAuthInterceptor(config?: InterceptorConfig): AuthInterceptor {
  if (!authInterceptorInstance) {
    const queue = getRequestQueue();
    
    // Use default configuration if not provided
    const defaultConfig: InterceptorConfig = {
      onAuthError: async (error: AuthError) => {
        console.log(`[Auth] Handling auth error: ${error.code} for command: ${error.command}`);
        
        try {
          // Call backend refresh token API
          await refreshActiveToken();
          console.log('[Auth] Token refresh successful');
        } catch (refreshError) {
          console.error('[Auth] Token refresh failed:', refreshError);
          throw refreshError;
        }
      },
      onRefreshSuccess: () => {
        console.log('[Auth] Refresh success callback triggered');
      },
      onRefreshFailed: () => {
        console.error('[Auth] Refresh failed callback triggered');
        
        // Trigger Reauth Dialog display
        if (showReauthDialogCallback) {
          console.log('[Auth] Triggering Reauth Dialog');
          showReauthDialogCallback();
        } else {
          console.warn('[Auth] Reauth Dialog callback not set');
        }
      },
    };
    
    authInterceptorInstance = new AuthInterceptor(
      config || defaultConfig,
      queue
    );
  }
  
  return authInterceptorInstance;
}

/**
 * Invoke function wrapped with Auth Interceptor.
 * 
 * Convenience function that automatically uses global Auth Interceptor instance
 * to execute Tauri commands with automatic authentication error handling.
 * 
 * @param command - Tauri command name
 * @param args - Command arguments
 * @returns Promise that resolves with command execution result
 * 
 * @example
 * ```typescript
 * import { authInvoke } from './auth';
 * 
 * // Use authInvoke instead of invoke for automatic auth error handling
 * const accounts = await authInvoke<Account[]>('get_accounts');
 * ```
 */
export async function authInvoke<T>(command: string, args?: any): Promise<T> {
  const interceptor = getAuthInterceptor();
  return interceptor.invoke<T>(command, args);
}

/**
 * Clear Request Queue.
 * 
 * Typically used during forced re-authentication.
 */
export function clearRequestQueue(): void {
  const queue = getRequestQueue();
  queue.clear();
}

/**
 * Get Request Queue length.
 * 
 * @returns Number of requests currently in queue
 */
export function getRequestQueueLength(): number {
  const queue = getRequestQueue();
  return queue.length;
}

/**
 * Reset Auth Interceptor and Request Queue instances.
 * 
 * Mainly used for testing or when re-initialization is needed.
 */
export function resetAuthInstances(): void {
  requestQueueInstance = null;
  authInterceptorInstance = null;
}

// Export classes and interfaces for advanced usage
export { AuthInterceptor, RequestQueue };
export type { InterceptorConfig, AuthError } from './interceptor';
export type { QueuedRequest } from './requestQueue';
