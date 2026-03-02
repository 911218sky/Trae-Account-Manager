import { useState } from "react";
import wxQrCode from "../assets/wx.jpg";
import logoImage from "../assets/logo.png";

export function About() {
  const [showImageModal, setShowImageModal] = useState(false);

  return (
    <div className="about-page">
      <div className="about-card">
        <div className="about-logo">
          <img src={logoImage} alt="Logo" className="about-logo-image" />
        </div>
        <h3>Trae Account Manager</h3>
        <p className="about-version">Version 1.0.0</p>
        <p className="about-desc">
          Trae account usage management tool to help you easily manage multiple Trae accounts.
        </p>
      </div>

      <div className="about-section">
        <h3>Features</h3>
        <ul className="feature-list">
          <li>📊 Multi-account usage statistics</li>
          <li>🔄 Real-time account data refresh</li>
          <li>📋 One-click copy account info</li>
          <li>🎨 Clean and beautiful interface</li>
        </ul>
      </div>

      <div className="about-section">
        <h3>Tech Stack</h3>
        <div className="tech-tags">
          <span className="tech-tag">Tauri</span>
          <span className="tech-tag">React</span>
          <span className="tech-tag">TypeScript</span>
          <span className="tech-tag">Rust</span>
        </div>
      </div>

      <div className="about-section">
        <h3>Support</h3>
        <p className="about-desc">
          If this tool is helpful to you, feel free to buy the author a coffee ☕
        </p>
        <div className="appreciation-container">
          <img
            src={wxQrCode}
            alt="WeChat QR Code"
            className="qr-code"
            onClick={() => setShowImageModal(true)}
          />
          <p className="appreciation-text">Click to enlarge · Scan with WeChat</p>
        </div>
      </div>

      {/* 图片放大模态框 */}
      {showImageModal && (
        <div className="image-modal-overlay" onClick={() => setShowImageModal(false)}>
          <div className="image-modal-content" onClick={(e) => e.stopPropagation()}>
            <button className="image-modal-close" onClick={() => setShowImageModal(false)}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="24" height="24">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
            <img src={wxQrCode} alt="WeChat QR Code" className="image-modal-img" />
            <p className="image-modal-text">Scan with WeChat</p>
          </div>
        </div>
      )}
    </div>
  );
}
