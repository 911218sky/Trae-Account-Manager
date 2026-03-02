import { useState } from "react";

interface UpdateTokenModalProps {
  isOpen: boolean;
  accountId: string;
  accountName: string;
  onClose: () => void;
  onUpdate: (accountId: string, token: string) => Promise<void>;
}

export function UpdateTokenModal({
  isOpen,
  accountId,
  accountName,
  onClose,
  onUpdate,
}: UpdateTokenModalProps) {
  const [inputValue, setInputValue] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  if (!isOpen) return null;

  // Extract token from input.
  // Supports multiple formats: JWT token, JSON response, or API response.
  const extractToken = (input: string): string | null => {
    const trimmed = input.trim();

    // Case 1: Direct JWT token (starts with eyJ)
    if (trimmed.startsWith("eyJ")) {
      return trimmed;
    }

    // Case 2: JSON response, attempt to parse
    try {
      const json = JSON.parse(trimmed);

      // GetUserToken API response format
      if (json.Result?.Token) {
        return json.Result.Token;
      }

      // Other possible formats
      if (json.token) {
        return json.token;
      }
      if (json.Token) {
        return json.Token;
      }
    } catch {
      // Not valid JSON, continue trying other methods
    }

    // Case 3: Try regex to extract token
    const tokenMatch = trimmed.match(/"Token"\s*:\s*"(eyJ[^"]+)"/);
    if (tokenMatch) {
      return tokenMatch[1];
    }

    // Case 4: Try to extract any JWT token (starts with eyJ)
    const jwtMatch = trimmed.match(/eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/);
    if (jwtMatch) {
      return jwtMatch[0];
    }

    return null;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputValue.trim()) {
      setError("Please enter a new token");
      return;
    }

    setLoading(true);
    setError("");

    try {
      const token = extractToken(inputValue);

      if (!token) {
        setError("Unable to recognize token. Please ensure you entered a valid token or GetUserToken API response");
        setLoading(false);
        return;
      }

      await onUpdate(accountId, token);
      setInputValue("");
      onClose();
    } catch (err: any) {
      setError(err.message || "Failed to update token");
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setError("");
    setInputValue("");
    onClose();
  };

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <h2>Update Token</h2>

        <p className="modal-desc">
          Update token for account <strong>{accountName}</strong>.
          <br />
          <small>Ensure the new token belongs to the same user, otherwise the update will fail.</small>
        </p>

        <div className="token-help">
          <details>
            <summary>How to get a new token?</summary>
            <ol>
              <li>Open <a href="https://www.trae.ai/account-setting#usage" target="_blank" rel="noopener noreferrer">trae.ai account settings page</a> and log in</li>
              <li>Press <kbd>F12</kbd> to open developer tools</li>
              <li>Switch to <strong>Network</strong> tab</li>
              <li>Refresh the page</li>
              <li>Find <code>GetUserToken</code> in the request list</li>
              <li>Click the request and find <strong>Response</strong> tab on the right</li>
              <li>Copy the entire response content and paste it below</li>
            </ol>
          </details>
        </div>

        <form onSubmit={handleSubmit}>
          <textarea
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            placeholder='Paste new token or API response...'
            rows={8}
            disabled={loading}
          />

          {error && <div className="error-message">{error}</div>}

          <div className="modal-actions">
            <button type="button" onClick={handleClose} disabled={loading}>
              Cancel
            </button>
            <button type="submit" className="primary" disabled={loading}>
              {loading ? "Updating..." : "Update Token"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
