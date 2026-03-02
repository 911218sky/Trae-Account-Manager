interface LoadingIndicatorProps {
  message?: string;
}

export function LoadingIndicator({ message = '載入中...' }: LoadingIndicatorProps) {
  return (
    <div className="loading-indicator">
      <div className="spinner">
        <div className="spinner-circle" />
      </div>
      <span className="loading-message">{message}</span>
    </div>
  );
}
