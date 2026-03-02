interface ConfirmModalProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  type?: "danger" | "warning" | "info";
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmModal({
  isOpen,
  title,
  message,
  confirmText = "Confirm",
  cancelText = "Cancel",
  type = "info",
  onConfirm,
  onCancel,
}: ConfirmModalProps) {
  if (!isOpen) return null;

  const icons = {
    danger: "🗑️",
    warning: "⚠️",
    info: "ℹ️",
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className={`confirm-modal confirm-${type}`} onClick={(e) => e.stopPropagation()}>
        <div className="confirm-icon">{icons[type]}</div>
        <h3 className="confirm-title">{title}</h3>
        <p className="confirm-message">{message}</p>
        <div className="confirm-actions">
          <button className="confirm-btn cancel" onClick={onCancel}>
            {cancelText}
          </button>
          <button className={`confirm-btn ${type}`} onClick={onConfirm}>
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
}
