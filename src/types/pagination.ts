export interface PaginationState {
  currentPage: number;
  pageSize: number;
  totalCount: number;
  hasMore: boolean;
  loading: boolean;
  error: Error | null;
}

export interface PaginatedResult<T> {
  data: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export interface CacheEntry<T> {
  data: T;
  timestamp: number;
  expiresAt: number;
}

export interface PerformanceMetrics {
  firstLoadTime: number;      // First load time (ms)
  fps: number;                 // Current FPS
  mainThreadBlockTime: number; // Main thread block time (ms)
  renderTime: number;          // Render time (ms)
  cacheHitRate: number;        // Cache hit rate (0-1)
}

export interface PerformanceConfig {
  pageSize: number;           // Number of accounts to load per batch, default 50
  bufferSize: number;         // Virtual scroll buffer size, default 5 items above and below
  cacheTimeout: number;       // Cache expiration time (milliseconds), default 300000 (5 minutes)
  triggerThreshold: number;   // Auto-load trigger point, default 0.8 (80%)
}

export const DEFAULT_PERFORMANCE_CONFIG: PerformanceConfig = {
  pageSize: 50,
  bufferSize: 5,
  cacheTimeout: 300000,
  triggerThreshold: 0.8,
};
