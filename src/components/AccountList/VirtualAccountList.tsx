import { useCallback, useRef, useEffect } from 'react';
import { AccountBrief, UsageSummary } from '../../types';
import { DEFAULT_PERFORMANCE_CONFIG } from '../../types/pagination';
import { AccountCard } from '../AccountCard';
import { AccountListItem } from '../AccountListItem';

interface VirtualAccountListProps {
  accounts: AccountBrief[];
  loading: boolean;
  hasMore: boolean;
  onLoadMore: () => void;
  viewMode: 'card' | 'list';
  selectedIds: Set<string>;
  onSelect: (id: string) => void;
  onContextMenu: (e: React.MouseEvent, id: string) => void;
  usageMap?: Map<string, UsageSummary>;
  bufferSize?: number;
  triggerThreshold?: number;
}

export function VirtualAccountList({
  accounts,
  loading,
  hasMore,
  onLoadMore,
  viewMode,
  selectedIds,
  onSelect,
  onContextMenu,
  usageMap = new Map(),
  triggerThreshold = DEFAULT_PERFORMANCE_CONFIG.triggerThreshold,
}: VirtualAccountListProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  // 處理滾動事件，觸發自動載入
  const handleScroll = useCallback(() => {
    if (loading || !hasMore) {
      return;
    }

    const container = containerRef.current;
    if (!container) return;

    const scrollHeight = container.scrollHeight;
    const clientHeight = container.clientHeight;
    const scrollTop = container.scrollTop;

    // 使用可配置的觸發閾值
    const scrollPercent = (scrollTop + clientHeight) / scrollHeight;

    if (scrollPercent >= triggerThreshold) {
      onLoadMore();
    }
  }, [loading, hasMore, onLoadMore, triggerThreshold]);

  // 監聽滾動事件
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    container.addEventListener('scroll', handleScroll);
    return () => container.removeEventListener('scroll', handleScroll);
  }, [handleScroll]);

  return (
    <div 
      ref={containerRef} 
      className={viewMode === 'card' ? 'account-grid' : 'account-list'}
      style={{ width: '100%', height: '100%', overflow: 'auto' }}
    >
      {accounts.map((account) => {
        const usage = usageMap.get(account.id) || null;
        const selected = selectedIds.has(account.id);

        return viewMode === 'card' ? (
          <AccountCard
            key={account.id}
            account={account}
            usage={usage}
            selected={selected}
            onSelect={onSelect}
            onContextMenu={onContextMenu}
          />
        ) : (
          <AccountListItem
            key={account.id}
            account={account}
            usage={usage}
            selected={selected}
            onSelect={onSelect}
            onContextMenu={onContextMenu}
          />
        );
      })}
      
      {loading && (
        <div className="loading-indicator">
          <div className="spinner" />
          <span>Loading...</span>
        </div>
      )}
    </div>
  );
}
