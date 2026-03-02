import { useEffect, useRef } from "react";

interface ContextMenuProps {
  x: number;
  y: number;
  onClose: () => void;
  onViewDetail: () => void;
  onRefresh: () => void;
  onUpdateToken: () => void;
  onCopyToken: () => void;
  onSwitchAccount: () => void;
  onClaimGift: () => void;
  onDelete: () => void;
  onRelogin: () => void;
  isCurrent?: boolean; // Whether this is the currently active account
}

export function ContextMenu({
  x,
  y,
  onClose,
  onViewDetail,
  onRefresh,
  onUpdateToken,
  onCopyToken,
  onSwitchAccount,
  onClaimGift,
  onDelete,
  onRelogin,
  isCurrent = false,
}: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // 调整菜单位置，防止超出屏幕
    if (menuRef.current) {
      const menu = menuRef.current;
      const rect = menu.getBoundingClientRect();

      if (rect.right > window.innerWidth) {
        menu.style.left = `${x - rect.width}px`;
      }
      if (rect.bottom > window.innerHeight) {
        menu.style.top = `${y - rect.height}px`;
      }
    }
  }, [x, y]);

  return (
    <>
      <div className="context-menu-overlay" onClick={onClose} />
      <div
        ref={menuRef}
        className="context-menu"
        style={{ left: x, top: y }}
      >
        <div className="context-menu-item" onClick={onViewDetail}>
          <span className="icon">👁</span>
          View Details
        </div>
        <div className="context-menu-item" onClick={onRefresh}>
          <span className="icon">🔄</span>
          Refresh Data
        </div>
        {isCurrent && (
          <div className="context-menu-item" onClick={onRelogin}>
            <span className="icon">🔐</span>
            Re-login
          </div>
        )}
        <div className="context-menu-item" onClick={onUpdateToken}>
          <span className="icon">🔐</span>
          Update Token
        </div>
        <div className="context-menu-item" onClick={onCopyToken}>
          <span className="icon">🔑</span>
          Copy Token
        </div>
        <div
          className={`context-menu-item ${isCurrent ? "disabled" : ""}`}
          onClick={isCurrent ? undefined : onSwitchAccount}
          title={isCurrent ? "Already using this account" : "Switch to this account"}
        >
          <span className="icon">{isCurrent ? "✓" : "🔀"}</span>
          {isCurrent ? "Currently Active" : "Switch Account"}
        </div>
        <div className="context-menu-item" onClick={onClaimGift}>
          <span className="icon">🎁</span>
          Claim Gift
        </div>
        <div className="context-menu-divider" />
        <div className="context-menu-item danger" onClick={onDelete}>
          <span className="icon">🗑</span>
          Delete Account
        </div>
      </div>
    </>
  );
}
