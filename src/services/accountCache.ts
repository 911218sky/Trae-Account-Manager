import { AccountBrief } from '../types';

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  expiresAt: number;
}

export class AccountCache {
  private cache: Map<string, CacheEntry<AccountBrief[]>>;
  private defaultTTL: number;

  constructor(defaultTTL: number = 300000) { // Default 5 minutes
    this.cache = new Map();
    this.defaultTTL = defaultTTL;
  }

  set(key: string, data: AccountBrief[], ttl?: number): void {
    try {
      const now = Date.now();
      const expiresAt = now + (ttl ?? this.defaultTTL);
      
      this.cache.set(key, {
        data,
        timestamp: now,
        expiresAt,
      });
    } catch (error) {
      console.error('[AccountCache] Cache write failed:', error);
      // Write failure does not affect functionality, handle silently
    }
  }

  get(key: string): AccountBrief[] | null {
    try {
      const entry = this.cache.get(key);
      
      if (!entry) {
        return null;
      }
      
      // Check if expired
      if (this.isExpired(key)) {
        this.cache.delete(key);
        return null;
      }
      
      return entry.data;
    } catch (error) {
      console.error('[AccountCache] Cache read failed, fallback to backend:', error);
      // Clear corrupted cache entry
      this.invalidate(key);
      return null;
    }
  }

  has(key: string): boolean {
    if (!this.cache.has(key)) {
      return false;
    }
    
    // Check if expired
    if (this.isExpired(key)) {
      this.cache.delete(key);
      return false;
    }
    
    return true;
  }

  invalidate(key: string): void {
    this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  isExpired(key: string): boolean {
    const entry = this.cache.get(key);
    
    if (!entry) {
      return true;
    }
    
    return Date.now() > entry.expiresAt;
  }

  // Clear all cache entries with specified prefix
  invalidateByPrefix(prefix: string): void {
    const keysToDelete: string[] = [];
    
    this.cache.forEach((_, key) => {
      if (key.startsWith(prefix)) {
        keysToDelete.push(key);
      }
    });
    
    keysToDelete.forEach((key) => this.cache.delete(key));
    
    console.log(`[AccountCache] Cleared ${keysToDelete.length} cache entries (prefix: ${prefix})`);
  }

  // Invalidate all account-related cache when accounts change
  invalidateAccountCache(): void {
    this.invalidateByPrefix('accounts-');
    console.log('[AccountCache] Cleared all account cache');
  }
}

// Singleton instance
export const accountCache = new AccountCache();
