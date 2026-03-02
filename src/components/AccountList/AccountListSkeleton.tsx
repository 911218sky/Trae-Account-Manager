interface AccountListSkeletonProps {
  count?: number;
  viewMode?: 'card' | 'list';
}

export function AccountListSkeleton({ count = 6, viewMode = 'card' }: AccountListSkeletonProps) {
  return (
    <div className={`account-list-skeleton ${viewMode}`}>
      {Array.from({ length: count }).map((_, index) => (
        <div key={index} className={`skeleton-item ${viewMode}`}>
          {viewMode === 'card' ? (
            <>
              <div className="skeleton-header">
                <div className="skeleton-checkbox" />
                <div className="skeleton-avatar" />
                <div className="skeleton-info">
                  <div className="skeleton-email" />
                  <div className="skeleton-name" />
                </div>
                <div className="skeleton-status" />
              </div>
              <div className="skeleton-tags">
                <div className="skeleton-tag" />
                <div className="skeleton-tag" />
              </div>
              <div className="skeleton-usage">
                <div className="skeleton-usage-header">
                  <div className="skeleton-label" />
                  <div className="skeleton-percent" />
                </div>
                <div className="skeleton-bar" />
                <div className="skeleton-numbers">
                  <div className="skeleton-number" />
                  <div className="skeleton-number" />
                </div>
              </div>
              <div className="skeleton-meta">
                <div className="skeleton-meta-item" />
                <div className="skeleton-meta-item" />
              </div>
            </>
          ) : (
            <>
              <div className="skeleton-checkbox" />
              <div className="skeleton-avatar" />
              <div className="skeleton-info">
                <div className="skeleton-email" />
                <div className="skeleton-id" />
              </div>
              <div className="skeleton-plan" />
              <div className="skeleton-usage-mini">
                <div className="skeleton-usage-text" />
                <div className="skeleton-bar-mini" />
              </div>
              <div className="skeleton-reset" />
              <div className="skeleton-status-dot" />
            </>
          )}
        </div>
      ))}
    </div>
  );
}
