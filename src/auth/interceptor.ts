import { invoke } from "@tauri-apps/api/core";
import { RequestQueue } from "./requestQueue";

/**
 * Authentication error interface.
 */
export interface AuthError {
  code: 401 | 403;
  message: string;
  command: string;
  args?: any;
  timestamp: number;
}

/**
 * Interceptor configuration interface.
 */
export interface InterceptorConfig {
  onAuthError: (error: AuthError) => Promise<void>;
  onRefreshSuccess: () => void;
  onRefreshFailed: () => void;
}

/**
 * Auth Interceptor class.
 * 
 * Intercepts all Tauri invoke call responses and automatically detects and handles
 * authentication failures (401/403 errors).
 * 
 * - Wraps Tauri invoke calls with automatic authentication error handling
 * - Detects 401/403 errors and triggers token refresh flow
 * - Uses isRefreshing flag to prevent concurrent refresh operations
 * - Uses refreshPromise to allow waiting requests to share same refresh flow
 * - Enqueues failed requests for retry after token update succeeds
 */
export class AuthInterceptor {
  private isRefreshing: boolean = false;
  private refreshPromise: Promise<void> | null = null;
  private config: InterceptorConfig;
  private requestQueue: RequestQueue;

  /**
   * Create Auth Interceptor instance.
   * 
   * @param config - Interceptor configuration
   * @param requestQueue - Request Queue instance
   */
  constructor(config: InterceptorConfig, requestQueue: RequestQueue) {
    this.config = config;
    this.requestQueue = requestQueue;
  }

  /**
   * Wrap Tauri invoke call with automatic authentication error handling.
   * 
   * @param command - Tauri command name
   * @param args - Command arguments
   * @returns Promise that resolves with command execution result
   */
  async invoke<T>(command: string, args?: any): Promise<T> {
    const startTime = performance.now();

    try {
      // Call Tauri invoke directly
      const result = await invoke<T>(command, args);
      
      // Log processing time (development mode only)
      const elapsed = performance.now() - startTime;
      if (elapsed > 50) {
        console.warn(`[AuthInterceptor] Slow interception: ${elapsed.toFixed(2)}ms for command: ${command}`);
      }
      
      return result;
    } catch (error: any) {
      // Check if this is an authentication error
      if (this.isAuthError(error)) {
        // Create AuthError object
        const authError: AuthError = {
          code: this.extractErrorCode(error),
          message: this.extractErrorMessage(error),
          command,
          args,
          timestamp: Date.now(),
        };

        // Log authentication error event
        console.log(`[AuthInterceptor] Auth error detected: ${authError.code} for command: ${command}`);

        // Handle authentication error
        await this.handleAuthError(authError);

        // Enqueue request for retry after token update
        return this.requestQueue.enqueue({
          command,
          args,
          resolve: () => {},
          reject: () => {},
        });
      }

      // Non-auth error: throw directly
      throw error;
    }
  }

  /**
   * Check if response is an authentication error.
   * 
   * @param error - Error object
   * @returns Whether this is an authentication error (401 or 403)
   */
  private isAuthError(error: any): boolean {
    // Tauri invoke errors may be string or object
    if (typeof error === 'string') {
      return error.includes('401') || error.includes('403') || 
             error.includes('Unauthorized') || error.includes('Forbidden');
    }
    
    if (error && typeof error === 'object') {
      // Check for status code in error object
      if (error.code === 401 || error.code === 403) {
        return true;
      }
      
      // Check error message
      const errorStr = JSON.stringify(error).toLowerCase();
      return errorStr.includes('401') || errorStr.includes('403') ||
             errorStr.includes('unauthorized') || errorStr.includes('forbidden');
    }
    
    return false;
  }

  /**
   * Extract error code from error object.
   * 
   * @param error - Error object
   * @returns Error code (401 or 403)
   */
  private extractErrorCode(error: any): 401 | 403 {
    if (typeof error === 'object' && error.code) {
      return error.code === 403 ? 403 : 401;
    }
    
    const errorStr = typeof error === 'string' ? error : JSON.stringify(error);
    return errorStr.includes('403') || errorStr.toLowerCase().includes('forbidden') ? 403 : 401;
  }

  /**
   * Extract error message from error object.
   * 
   * @param error - Error object
   * @returns Error message
   */
  private extractErrorMessage(error: any): string {
    if (typeof error === 'string') {
      return error;
    }
    
    if (error && typeof error === 'object') {
      return error.message || error.error || JSON.stringify(error);
    }
    
    return 'Unknown authentication error';
  }

  /**
   * Trigger token refresh flow.
   * 
   * Uses isRefreshing flag and refreshPromise to ensure concurrent requests
   * only execute refresh once.
   * 
   * @param error - Authentication error object
   */
  private async handleAuthError(error: AuthError): Promise<void> {
    // If already refreshing, wait for existing refresh to complete
    if (this.isRefreshing && this.refreshPromise) {
      console.log(`[AuthInterceptor] Refresh already in progress, waiting...`);
      await this.refreshPromise;
      return;
    }

    // Set isRefreshing flag to prevent concurrent refresh
    this.isRefreshing = true;

    // Create refresh promise so other waiting requests can share it
    this.refreshPromise = (async () => {
      try {
        console.log(`[AuthInterceptor] Starting token refresh...`);
        
        // Call configured onAuthError callback to trigger actual refresh flow
        await this.config.onAuthError(error);
        
        console.log(`[AuthInterceptor] Token refresh successful`);
        
        // Notify refresh success
        this.config.onRefreshSuccess();
        
        // Process queued requests
        await this.requestQueue.processQueue();
      } catch (refreshError) {
        console.error(`[AuthInterceptor] Token refresh failed:`, refreshError);
        
        // Notify refresh failed
        this.config.onRefreshFailed();
        
        throw refreshError;
      } finally {
        // Reset state
        this.isRefreshing = false;
        this.refreshPromise = null;
      }
    })();

    // Wait for refresh to complete
    await this.refreshPromise;
  }
}
