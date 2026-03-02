/**
 * WebSocket Client for IDE-side communication
 * 
 * Connects to the WebSocket server (default: ws://localhost:9527)
 * Implements exponential backoff reconnection strategy
 * Handles heartbeat messages and responds with HeartbeatAck
 */

export interface WebSocketConfig {
  url?: string;
  port?: number;
  maxRetries?: number;
  baseDelayMs?: number;
  maxDelayMs?: number;
  heartbeatTimeoutMs?: number;
}

export interface SessionChangedEvent {
  type: 'session_changed';
  account_id: string;
  user_id: string;
  email: string;
  token: string;
  timestamp: number;
}

export interface HeartbeatEvent {
  type: 'heartbeat';
  timestamp: number;
}

export interface HeartbeatAckEvent {
  type: 'heartbeat_ack';
  client_id: string;
  timestamp: number;
}

export interface ErrorEvent {
  type: 'error';
  code: string;
  message: string;
}

export type WebSocketEvent = SessionChangedEvent | HeartbeatEvent | HeartbeatAckEvent | ErrorEvent;

export type SessionChangedCallback = (event: SessionChangedEvent) => void;
export type ConnectionStatusCallback = (connected: boolean) => void;
export type ErrorCallback = (error: Error) => void;

/**
 * WebSocket client with automatic reconnection
 */
export class WebSocketClient {
  private config: Required<WebSocketConfig>;
  private ws: WebSocket | null = null;
  private clientId: string | null = null;
  private retryCount: number = 0;
  private reconnectTimeout: number | null = null;
  private heartbeatTimeout: number | null = null;
  private isManualClose: boolean = false;
  
  // Callbacks
  private sessionChangedCallbacks: Set<SessionChangedCallback> = new Set();
  private connectionStatusCallbacks: Set<ConnectionStatusCallback> = new Set();
  private errorCallbacks: Set<ErrorCallback> = new Set();

  constructor(config: WebSocketConfig = {}) {
    this.config = {
      url: config.url || 'ws://localhost',
      port: config.port || 9527,
      maxRetries: config.maxRetries || 10,
      baseDelayMs: config.baseDelayMs || 1000,
      maxDelayMs: config.maxDelayMs || 30000,
      heartbeatTimeoutMs: config.heartbeatTimeoutMs || 35000, // 35s (server timeout is 30s)
    };
  }

  /**
   * Connect to the WebSocket server
   */
  public connect(): void {
    if (this.ws && (this.ws.readyState === WebSocket.CONNECTING || this.ws.readyState === WebSocket.OPEN)) {
      console.log('[WebSocketClient] Already connected or connecting');
      return;
    }

    this.isManualClose = false;
    const url = `${this.config.url}:${this.config.port}`;
    
    try {
      console.log(`[WebSocketClient] Connecting to ${url}...`);
      this.ws = new WebSocket(url);
      
      this.ws.onopen = this.handleOpen.bind(this);
      this.ws.onmessage = this.handleMessage.bind(this);
      this.ws.onerror = this.handleError.bind(this);
      this.ws.onclose = this.handleClose.bind(this);
    } catch (error) {
      console.error('[WebSocketClient] Failed to create WebSocket:', error);
      this.scheduleReconnect();
    }
  }

  /**
   * Disconnect from the WebSocket server
   */
  public disconnect(): void {
    console.log('[WebSocketClient] Disconnecting...');
    this.isManualClose = true;
    this.clearReconnectTimeout();
    this.clearHeartbeatTimeout();
    
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    
    this.clientId = null;
    this.retryCount = 0;
    this.notifyConnectionStatus(false);
  }

  /**
   * Check if the client is connected
   */
  public isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  /**
   * Register a callback for session_changed events
   */
  public onSessionChanged(callback: SessionChangedCallback): () => void {
    this.sessionChangedCallbacks.add(callback);
    return () => this.sessionChangedCallbacks.delete(callback);
  }

  /**
   * Register a callback for connection status changes
   */
  public onConnectionStatus(callback: ConnectionStatusCallback): () => void {
    this.connectionStatusCallbacks.add(callback);
    return () => this.connectionStatusCallbacks.delete(callback);
  }

  /**
   * Register a callback for errors
   */
  public onError(callback: ErrorCallback): () => void {
    this.errorCallbacks.add(callback);
    return () => this.errorCallbacks.delete(callback);
  }

  /**
   * Handle WebSocket open event
   */
  private handleOpen(): void {
    console.log('[WebSocketClient] Connected successfully');
    this.retryCount = 0;
    this.notifyConnectionStatus(true);
    this.resetHeartbeatTimeout();
  }

  /**
   * Handle incoming WebSocket messages
   */
  private handleMessage(event: MessageEvent): void {
    try {
      const message: WebSocketEvent = JSON.parse(event.data);
      
      switch (message.type) {
        case 'session_changed':
          console.log('[WebSocketClient] Received session_changed event:', message);
          this.notifySessionChanged(message);
          break;
          
        case 'heartbeat':
          console.log('[WebSocketClient] Received heartbeat');
          this.handleHeartbeat(message);
          this.resetHeartbeatTimeout();
          break;
          
        case 'error':
          console.error('[WebSocketClient] Received error:', message);
          this.notifyError(new Error(`${message.code}: ${message.message}`));
          break;
          
        default:
          console.warn('[WebSocketClient] Unknown message type:', message);
      }
    } catch (error) {
      console.error('[WebSocketClient] Failed to parse message:', error);
      this.notifyError(error as Error);
    }
  }

  /**
   * Handle WebSocket error event
   */
  private handleError(event: Event): void {
    console.error('[WebSocketClient] WebSocket error:', event);
    const error = new Error('WebSocket connection error');
    this.notifyError(error);
  }

  /**
   * Handle WebSocket close event
   */
  private handleClose(event: CloseEvent): void {
    console.log(`[WebSocketClient] Connection closed (code: ${event.code}, reason: ${event.reason})`);
    this.clearHeartbeatTimeout();
    this.notifyConnectionStatus(false);
    
    if (!this.isManualClose) {
      this.scheduleReconnect();
    }
  }

  /**
   * Handle heartbeat message and send acknowledgment
   */
  private handleHeartbeat(_message: HeartbeatEvent): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      return;
    }

    // Generate client ID if not set (first heartbeat)
    if (!this.clientId) {
      this.clientId = this.generateClientId();
    }

    const ack: HeartbeatAckEvent = {
      type: 'heartbeat_ack',
      client_id: this.clientId,
      timestamp: Date.now(),
    };

    try {
      this.ws.send(JSON.stringify(ack));
      console.log('[WebSocketClient] Sent heartbeat_ack');
    } catch (error) {
      console.error('[WebSocketClient] Failed to send heartbeat_ack:', error);
    }
  }

  /**
   * Schedule reconnection with exponential backoff
   */
  private scheduleReconnect(): void {
    if (this.retryCount >= this.config.maxRetries) {
      console.error(`[WebSocketClient] Max retries (${this.config.maxRetries}) reached, giving up`);
      const error = new Error('Max reconnection attempts reached');
      this.notifyError(error);
      return;
    }

    // Calculate delay with exponential backoff: baseDelay * 2^retryCount
    const delay = Math.min(
      this.config.baseDelayMs * Math.pow(2, this.retryCount),
      this.config.maxDelayMs
    );

    this.retryCount++;
    console.log(`[WebSocketClient] Reconnecting in ${delay}ms (attempt ${this.retryCount}/${this.config.maxRetries})...`);

    this.clearReconnectTimeout();
    this.reconnectTimeout = window.setTimeout(() => {
      this.connect();
    }, delay);
  }

  /**
   * Clear reconnect timeout
   */
  private clearReconnectTimeout(): void {
    if (this.reconnectTimeout !== null) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
  }

  /**
   * Reset heartbeat timeout
   */
  private resetHeartbeatTimeout(): void {
    this.clearHeartbeatTimeout();
    
    this.heartbeatTimeout = window.setTimeout(() => {
      console.warn('[WebSocketClient] Heartbeat timeout, connection may be dead');
      if (this.ws) {
        this.ws.close();
      }
    }, this.config.heartbeatTimeoutMs);
  }

  /**
   * Clear heartbeat timeout
   */
  private clearHeartbeatTimeout(): void {
    if (this.heartbeatTimeout !== null) {
      clearTimeout(this.heartbeatTimeout);
      this.heartbeatTimeout = null;
    }
  }

  /**
   * Generate a unique client ID
   */
  private generateClientId(): string {
    return `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Notify all session_changed callbacks
   */
  private notifySessionChanged(event: SessionChangedEvent): void {
    this.sessionChangedCallbacks.forEach(callback => {
      try {
        callback(event);
      } catch (error) {
        console.error('[WebSocketClient] Error in session_changed callback:', error);
      }
    });
  }

  /**
   * Notify all connection status callbacks
   */
  private notifyConnectionStatus(connected: boolean): void {
    this.connectionStatusCallbacks.forEach(callback => {
      try {
        callback(connected);
      } catch (error) {
        console.error('[WebSocketClient] Error in connection status callback:', error);
      }
    });
  }

  /**
   * Notify all error callbacks
   */
  private notifyError(error: Error): void {
    this.errorCallbacks.forEach(callback => {
      try {
        callback(error);
      } catch (error) {
        console.error('[WebSocketClient] Error in error callback:', error);
      }
    });
  }
}

/**
 * Singleton instance for global use
 */
let globalClient: WebSocketClient | null = null;

/**
 * Get or create the global WebSocket client instance
 */
export function getWebSocketClient(config?: WebSocketConfig): WebSocketClient {
  if (!globalClient) {
    globalClient = new WebSocketClient(config);
  }
  return globalClient;
}

/**
 * Reset the global WebSocket client instance (useful for testing)
 */
export function resetWebSocketClient(): void {
  if (globalClient) {
    globalClient.disconnect();
    globalClient = null;
  }
}
