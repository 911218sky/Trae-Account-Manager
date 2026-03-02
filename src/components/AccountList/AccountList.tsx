import { useState, useCallback } from 'react';
import { usePaginatedAccounts } from '../../hooks/usePaginatedAccounts';
import { VirtualAccountList } from './VirtualAccountList';
import { AccountListSkeleton } from './AccountListSkeleton';
import { LoadingIndicator } from './LoadingIndicator';
import { AccountBrief, UsageSummary } from '../../types';
import { PerformanceConfig, DEFAULT_PERFORMANCE_CONFIG } from '../../types/pagination';

interface AccountListProps {
  viewMode?: 'card' | 'list';
  selectedIds?: Set<string>;
  accounts?: AccountBrief[]; // 從外部傳入帳號數據
  onAccountSelect?: (accountId: string) => void;
  onAccountContextMenu?: (e: React.MouseEvent, accountId: string) => void;
  usageMap?: Map<string, UsageSummary>;
  config?: Partial<PerformanceConfig>;
}

export function AccountList({
  viewMode = 'card',
  selectedIds: externalSelectedIds,
  accounts: externalAccounts,
  onAccountSelect,
  onAccountContextMenu,
  usageMap = new Map(),
  config,
}: AccountListProps) {
  const [internalSelectedIds, setInternalSelectedIds] = useState<Set<string>>(new Set());
  
  // 使用外部傳入的 selectedIds，如果沒有則使用內部狀態
  const selectedIds = externalSelectedIds || internalSelectedIds;
  
  // 合併配置
  const finalConfig = {
    ...DEFAULT_PERFORMANCE_CONFIG,
    ...config,
  };
  
  // 如果有外部傳入的帳號，直接使用；否則使用分頁加載
  const shouldUsePagination = !externalAccounts;
  
  const {
    accounts: paginatedAccounts,
    loading: paginatedLoading,
    error: paginatedError,
    hasMore,
    loadMore,
    refresh,
    totalCount,
  } = usePaginatedAccounts({
    pageSize: finalConfig.pageSize,
    triggerThreshold: finalConfig.triggerThreshold,
    cacheTimeout: finalConfig.cacheTimeout,
  });

  // 使用外部帳號或分頁帳號
  const accounts = externalAccounts || paginatedAccounts;
  const loading = shouldUsePagination ? paginatedLoading : false;
  const error = shouldUsePagination ? paginatedError : null;

  const handleSelect = useCallback((id: string) => {
    // 如果沒有外部 selectedIds，更新內部狀態
    if (!externalSelectedIds) {
      setInternalSelectedIds((prev) => {
        const next = new Set(prev);
        if (next.has(id)) {
          next.delete(id);
        } else {
          next.add(id);
        }
        return next;
      });
    }
    
    // 總是調用外部回調
    onAccountSelect?.(id);
  }, [externalSelectedIds, onAccountSelect]);

  const handleContextMenu = useCallback((e: React.MouseEvent, id: string) => {
    e.preventDefault();
    onAccountContextMenu?.(e, id);
  }, [onAccountContextMenu]);

  const handleRetry = useCallback(() => {
    refresh();
  }, [refresh]);

  // 首次載入中，顯示骨架屏
  if (loading && accounts.length === 0) {
    return <AccountListSkeleton count={6} viewMode={viewMode} />;
  }

  // 載入錯誤，顯示錯誤訊息與重試按鈕
  if (error && accounts.length === 0) {
    return (
      <div className="account-list-error">
        <div className="error-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
        </div>
        <h3 className="error-title">Load Failed</h3>
        <p className="error-message">{error.message}</p>
        <button className="retry-button" onClick={handleRetry}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          Retry
        </button>
      </div>
    );
  }

  // 無帳號資料
  if (accounts.length === 0) {
    return (
      <div className="account-list-empty">
        <div className="empty-icon">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
            <circle cx="12" cy="7" r="4"/>
          </svg>
        </div>
        <h3 className="empty-title">No Accounts</h3>
        <p className="empty-message">Click the "Add Account" button in the top right to add your first account</p>
      </div>
    );
  }

  return (
    <div className="account-list-container">
      <div className="account-list-header">
        <div className="account-count">
          Total {externalAccounts ? externalAccounts.length : totalCount} accounts
          {selectedIds.size > 0 && ` · ${selectedIds.size} selected`}
        </div>
        {error && (
          <div className="load-error-banner">
            <span>Failed to load more: {error.message}</span>
            <button onClick={handleRetry}>Retry</button>
          </div>
        )}
      </div>

      <VirtualAccountList
        accounts={accounts}
        loading={loading}
        hasMore={shouldUsePagination ? hasMore : false}
        onLoadMore={shouldUsePagination ? loadMore : () => {}}
        viewMode={viewMode}
        selectedIds={selectedIds}
        onSelect={handleSelect}
        onContextMenu={handleContextMenu}
        usageMap={usageMap}
        bufferSize={finalConfig.bufferSize}
        triggerThreshold={finalConfig.triggerThreshold}
      />

      {loading && accounts.length > 0 && (
        <div className="bottom-loading">
          <LoadingIndicator message="Loading more..." />
        </div>
      )}
    </div>
  );
}
