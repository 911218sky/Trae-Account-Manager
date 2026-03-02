import { useState, useCallback, useRef, useEffect } from 'react';
import { getAccountsPaginated } from '../api';
import { AccountBrief } from '../types';
import { accountCache } from '../services/accountCache';

export interface UsePaginatedAccountsOptions {
  pageSize?: number;
  triggerThreshold?: number;
  cacheTimeout?: number;
}

export interface UsePaginatedAccountsReturn {
  accounts: AccountBrief[];
  loading: boolean;
  error: Error | null;
  hasMore: boolean;
  loadMore: () => Promise<void>;
  refresh: () => Promise<void>;
  totalCount: number;
}

export function usePaginatedAccounts(
  options: UsePaginatedAccountsOptions = {}
): UsePaginatedAccountsReturn {
  const {
    pageSize = 50,
    cacheTimeout = 300000, // 5 分鐘
  } = options;

  const [accounts, setAccounts] = useState<AccountBrief[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [hasMore, setHasMore] = useState(true);
  const [totalCount, setTotalCount] = useState(0);
  const currentPageRef = useRef(0);
  const isLoadingRef = useRef(false);

  const loadPage = useCallback(
    async (page: number, append: boolean = true) => {
      // 防止重複載入
      if (isLoadingRef.current) {
        return;
      }

      isLoadingRef.current = true;
      setLoading(true);
      setError(null);

      try {
        const cacheKey = `accounts-page-${page}`;
        
        // 優先從快取讀取
        const cachedData = accountCache.get(cacheKey);
        if (cachedData && append) {
          setAccounts((prev) => [...prev, ...cachedData]);
          currentPageRef.current = page;
          return;
        }

        // 從後端載入
        const result = await getAccountsPaginated(page, pageSize);
        
        // 儲存至快取
        accountCache.set(cacheKey, result.data, cacheTimeout);
        
        // 更新狀態
        if (append) {
          setAccounts((prev) => [...prev, ...result.data]);
        } else {
          setAccounts(result.data);
        }
        
        setHasMore(result.has_more);
        setTotalCount(result.total);
        currentPageRef.current = page;
      } catch (err) {
        const error = err instanceof Error ? err : new Error('載入帳號失敗');
        setError(error);
        console.error('[usePaginatedAccounts] 載入失敗:', error);
      } finally {
        setLoading(false);
        isLoadingRef.current = false;
      }
    },
    [pageSize, cacheTimeout]
  );

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) {
      return;
    }
    
    const nextPage = currentPageRef.current + 1;
    await loadPage(nextPage, true);
  }, [hasMore, loading, loadPage]);

  const refresh = useCallback(async () => {
    // 清除快取
    accountCache.clear();
    
    // 重置狀態
    setAccounts([]);
    setHasMore(true);
    setTotalCount(0);
    currentPageRef.current = 0;
    
    // 載入第一頁
    await loadPage(1, false);
  }, [loadPage]);

  // 初始載入
  useEffect(() => {
    loadPage(1, false);
  }, [loadPage]);

  return {
    accounts,
    loading,
    error,
    hasMore,
    loadMore,
    refresh,
    totalCount,
  };
}
