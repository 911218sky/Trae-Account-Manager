import { invoke } from "@tauri-apps/api/core";

/**
 * Queued request item in the request queue.
 */
export interface QueuedRequest {
  id: string;
  command: string;
  args: any;
  resolve: (value: any) => void;
  reject: (error: any) => void;
  timestamp: number;
}

/**
 * Request Queue class.
 * 
 * Manages HTTP request queue for failed authentication requests.
 * Retries all requests in FIFO order after token update succeeds.
 * 
 * - Enqueue failed requests and return Promise
 * - Retry all queued requests with new token
 * - Clear queue
 * - Query queue length
 */
export class RequestQueue {
  private queue: QueuedRequest[] = [];
  private isProcessing: boolean = false;
  private requestIdCounter: number = 0;

  /**
   * Enqueue a request for retry.
   * 
   * @param request - Request info (without id and timestamp)
   * @returns Promise that resolves or rejects when retry completes
   */
  enqueue(request: Omit<QueuedRequest, 'id' | 'timestamp'>): Promise<any> {
    return new Promise((resolve, reject) => {
      const queuedRequest: QueuedRequest = {
        id: `req_${++this.requestIdCounter}_${Date.now()}`,
        command: request.command,
        args: request.args,
        resolve,
        reject,
        timestamp: Date.now(),
      };

      this.queue.push(queuedRequest);
    });
  }

  /**
   * Process all queued requests.
   * 
   * Retries all suspended requests with new access token in FIFO order.
   * 
   * - Passes response to original caller on success
   * - Re-enqueues on 401/403 errors
   * - Passes other errors to original caller
   */
  async processQueue(): Promise<void> {
    if (this.isProcessing) {
      return;
    }

    this.isProcessing = true;

    try {
      // Copy queue and clear original to avoid reprocessing new requests
      const requestsToProcess = [...this.queue];
      this.queue = [];

      // Process each request in FIFO order
      for (const request of requestsToProcess) {
        try {
          // Retry request with new token
          const result = await invoke(request.command, request.args);
          
          // Success: pass response to original caller
          request.resolve(result);
        } catch (error: any) {
          // Check if this is an authentication error
          const isAuthError = this.isAuthError(error);
          
          if (isAuthError) {
            // Auth error: re-enqueue request for next refresh
            // Note: actual refresh is triggered by Auth Interceptor
            this.queue.push(request);
          } else {
            // Non-auth error: pass error to original caller
            request.reject(error);
          }
        }
      }
    } finally {
      this.isProcessing = false;
    }
  }

  /**
   * Clear the queue.
   * 
   * Clears all suspended requests, typically used during forced re-authentication.
   */
  clear(): void {
    // Reject all queued requests
    for (const request of this.queue) {
      request.reject(new Error('Request queue cleared due to force reauth'));
    }
    
    this.queue = [];
  }

  /**
   * Get queue length.
   * 
   * @returns Number of requests currently in queue
   */
  get length(): number {
    return this.queue.length;
  }

  /**
   * Check if error is an authentication error (401/403).
   * 
   * @param error - Error object
   * @returns Whether this is an authentication error
   */
  private isAuthError(error: any): boolean {
    // Tauri invoke errors may contain status code information
    // Adjust logic based on actual error format
    if (typeof error === 'string') {
      return error.includes('401') || error.includes('403') || 
             error.includes('Unauthorized') || error.includes('Forbidden');
    }
    
    if (error && typeof error === 'object') {
      const errorStr = JSON.stringify(error).toLowerCase();
      return errorStr.includes('401') || errorStr.includes('403') ||
             errorStr.includes('unauthorized') || errorStr.includes('forbidden');
    }
    
    return false;
  }
}
