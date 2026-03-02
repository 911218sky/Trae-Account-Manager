import { useState, useEffect } from 'react';
import { startBrowserLogin } from '../api';

interface ReauthDialogProps {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export function ReauthDialog({ open, onClose, onSuccess }: ReauthDialogProps) {
  const [error, setError] = useState<string | null>(null);
  const [isLoggingIn, setIsLoggingIn] = useState(false);

  // Auto-hide error message after 5 seconds
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => {
        setError(null);
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  // Listen for login success event
  useEffect(() => {
    if (!open) return;

    const handleLoginSuccess = () => {
      setIsLoggingIn(false);
      setError(null);
      onSuccess();
    };

    const handleLoginFailed = (message: string) => {
      setIsLoggingIn(false);
      setError(message || 'Login failed, please try again');
    };

    const handleLoginCancelled = () => {
      setIsLoggingIn(false);
    };

    // Use Tauri event listener
    import('@tauri-apps/api/event').then(({ listen }) => {
      const unlisten1 = listen('login-success', handleLoginSuccess);
      const unlisten2 = listen('login-failed', (event: any) => handleLoginFailed(event.payload));
      const unlisten3 = listen('login-cancelled', handleLoginCancelled);

      return () => {
        unlisten1.then(fn => fn());
        unlisten2.then(fn => fn());
        unlisten3.then(fn => fn());
      };
    });
  }, [open, onSuccess]);

  const handleBrowserLogin = async () => {
    try {
      setIsLoggingIn(true);
      setError(null);
      await startBrowserLogin();
    } catch (err) {
      setIsLoggingIn(false);
      setError(err instanceof Error ? err.message : 'Unable to open login window');
    }
  };

  const handleCancel = () => {
    if (!isLoggingIn) {
      setError(null);
      onClose();
    }
  };

  if (!open) return null;

  return (
    <div className="modal-overlay" onClick={handleCancel}>
      <div className="reauth-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="reauth-icon">🔐</div>
        <h3 className="reauth-title">Login Expired</h3>
        <p className="reauth-message">
          Your login session has expired. Please log in again to continue.
        </p>

        {error && (
          <div className="reauth-error">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            <span>{error}</span>
          </div>
        )}

        <div className="reauth-actions">
          <button 
            className="reauth-btn cancel" 
            onClick={handleCancel}
            disabled={isLoggingIn}
          >
            Cancel
          </button>
          <button 
            className="reauth-btn primary" 
            onClick={handleBrowserLogin}
            disabled={isLoggingIn}
          >
            {isLoggingIn ? (
              <>
                <svg className="spinner" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
                </svg>
                Logging in...
              </>
            ) : (
              'Log In Again'
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
