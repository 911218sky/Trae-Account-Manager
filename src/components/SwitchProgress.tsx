import { useEffect, useState } from "react";

export interface SwitchProgressData {
  step: number;
  total: number;
  message: string;
}

interface SwitchProgressProps {
  progress: SwitchProgressData | null;
  onClose?: () => void;
}

export function SwitchProgress({ progress, onClose }: SwitchProgressProps) {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    if (progress) {
      setIsVisible(true);
    } else {
      setIsVisible(false);
    }
  }, [progress]);

  if (!isVisible || !progress) {
    return null;
  }

  const percentage = (progress.step / progress.total) * 100;

  return (
    <div className="switch-progress-overlay">
      <div className="switch-progress-modal">
        <div className="switch-progress-header">
          <h3>Switching Account</h3>
          {onClose && (
            <button className="switch-progress-close" onClick={onClose}>
              ×
            </button>
          )}
        </div>
        <div className="switch-progress-content">
          <div className="switch-progress-spinner">
            <div className="spinner-ring"></div>
          </div>
          <div className="switch-progress-info">
            <p className="switch-progress-message">{progress.message}</p>
            <p className="switch-progress-step">
              Step {progress.step} / {progress.total}
            </p>
          </div>
          <div className="switch-progress-bar">
            <div
              className="switch-progress-bar-fill"
              style={{ width: `${percentage}%` }}
            ></div>
          </div>
        </div>
      </div>
    </div>
  );
}
