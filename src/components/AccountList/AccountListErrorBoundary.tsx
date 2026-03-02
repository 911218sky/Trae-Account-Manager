import React, { Component, ReactNode } from 'react';
import { errorMonitor } from '../../services/errorMonitor';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class AccountListErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    // 記錄錯誤到 ErrorMonitor
    errorMonitor.logError('render', error, {
      componentStack: errorInfo.componentStack,
    });
  }

  handleRetry = (): void => {
    this.setState({
      hasError: false,
      error: null,
    });
  };

  render(): ReactNode {
    if (this.state.hasError) {
      return (
        <div className="error-boundary-fallback">
          <div className="error-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
          </div>
          <h3 className="error-title">渲染錯誤</h3>
          <p className="error-message">
            {this.state.error?.message || '帳號列表渲染時發生錯誤'}
          </p>
          <div className="error-actions">
            <button className="retry-button" onClick={this.handleRetry}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
              </svg>
              重新載入
            </button>
            <button 
              className="reload-button" 
              onClick={() => window.location.reload()}
            >
              重新整理頁面
            </button>
          </div>
          {import.meta.env.DEV && this.state.error && (
            <details className="error-details">
              <summary>錯誤詳情（開發模式）</summary>
              <pre>{this.state.error.stack}</pre>
            </details>
          )}
        </div>
      );
    }

    return this.props.children;
  }
}
