import { useState } from "react";

interface SwitchErrorModalProps {
  isOpen: boolean;
  errorMessage: string;
  accountId: string;
  onRetry: (accountId: string) => void;
  onRollback: () => void;
  onClose: () => void;
}

export function SwitchErrorModal({
  isOpen,
  errorMessage,
  accountId,
  onRetry,
  onRollback,
  onClose,
}: SwitchErrorModalProps) {
  const [isRetrying, setIsRetrying] = useState(false);
  const [isRollingBack, setIsRollingBack] = useState(false);

  if (!isOpen) {
    return null;
  }

  const handleRetry = async () => {
    setIsRetrying(true);
    try {
      await onRetry(accountId);
    } finally {
      setIsRetrying(false);
    }
  };

  const handleRollback = async () => {
    setIsRollingBack(true);
    try {
      await onRollback();
    } finally {
      setIsRollingBack(false);
    }
  };

  return (
    <div className="switch-error-overlay" onClick={onClose}>
      <div className="switch-error-modal" onClick={(e) => e.stopPropagation()}>
        <div className="switch-error-header">
          <div className="switch-error-icon">⚠️</div>
          <h3>Account Switch Failed</h3>
        </div>
        <div className="switch-error-content">
          <p className="switch-error-message">{errorMessage}</p>
          <div className="switch-error-actions">
            <button
              className="switch-error-btn retry"
              onClick={handleRetry}
              disabled={isRetrying || isRollingBack}
            >
              {isRetrying ? (
                <>
                  <span className="btn-spinner"></span>
                  Retrying...
                </>
              ) : (
                <>
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    width="16"
                    height="16"
                  >
                    <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
                  </svg>
                  Retry
                </>
              )}
            </button>
            <button
              className="switch-error-btn rollback"
              onClick={handleRollback}
              disabled={isRetrying || isRollingBack}
            >
              {isRollingBack ? (
                <>
                  <span className="btn-spinner"></span>
                  Rolling back...
                </>
              ) : (
                <>
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    width="16"
                    height="16"
                  >
                    <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
                    <polyline points="9 22 9 12 15 12 15 22" />
                  </svg>
                  Rollback to Previous Account
                </>
              )}
            </button>
            <button
              className="switch-error-btn cancel"
              onClick={onClose}
              disabled={isRetrying || isRollingBack}
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
