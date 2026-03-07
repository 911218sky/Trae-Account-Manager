---
inclusion: fileMatch
fileMatchPattern: 'src-tauri/**/*.rs'
name: backend-rust
description: Rust and Tauri development guidelines. Loaded when working with backend code.
---

# Backend Development Guidelines (Rust/Tauri)

## Rust Conventions

### Naming Conventions
- Functions and variables: `snake_case`
- Types and traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

```rust
const MAX_RETRIES: u32 = 3;

struct AccountManager {
    accounts: Vec<Account>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self { accounts: Vec::new() }
    }
    
    pub fn add_account(&mut self, account: Account) -> Result<()> {
        // Implementation
    }
}
```

### Error Handling
Always use `Result<T, E>` for operations that can fail:

```rust
use anyhow::{Context, Result};

fn fetch_account(id: &str) -> Result<Account> {
    let account = database::get_account(id)
        .context("Failed to fetch account from database")?;
    
    Ok(account)
}

// For Tauri commands, convert to String error
#[tauri::command]
async fn get_account(id: String) -> Result<Account, String> {
    fetch_account(&id)
        .map_err(|e| e.to_string())
}
```

### Ownership and Borrowing
- Prefer `&str` over `String` for function parameters
- Use `&` for read-only access
- Use `&mut` for mutable access
- Use `clone()` only when necessary

```rust
// Good: Borrow string slice
fn process_name(name: &str) -> String {
    name.to_uppercase()
}

// Good: Borrow struct
fn validate_account(account: &Account) -> bool {
    !account.name.is_empty()
}

// Good: Mutable borrow
fn update_account(account: &mut Account, new_name: String) {
    account.name = new_name;
}
```

### Async/Await
Use `async`/`await` for I/O operations:

```rust
use tokio::fs;

async fn read_config() -> Result<Config> {
    let content = fs::read_to_string("config.json").await?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

#[tauri::command]
async fn load_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, String> {
    let accounts = state.account_manager.lock().await.get_all().await
        .map_err(|e| e.to_string())?;
    Ok(accounts)
}
```

## Tauri Commands

### Basic Command
```rust
#[tauri::command]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// Register in main.rs
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Command with State
```rust
use tauri::State;

struct AppState {
    account_manager: Arc<Mutex<AccountManager>>,
}

#[tauri::command]
async fn add_account(
    name: String,
    token: String,
    state: State<'_, AppState>
) -> Result<Account, String> {
    let mut manager = state.account_manager.lock().await;
    manager.add_account(name, token)
        .await
        .map_err(|e| e.to_string())
}
```

### Command with Window
```rust
use tauri::Window;

#[tauri::command]
async fn close_window(window: Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}
```

### Emitting Events
```rust
use tauri::Manager;

#[tauri::command]
async fn switch_account(
    account_id: String,
    app: tauri::AppHandle
) -> Result<(), String> {
    // Perform switch
    let result = perform_switch(&account_id).await?;
    
    // Emit event to frontend
    app.emit_all("account_switched", result)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}
```

## Common Patterns

### Secure Storage (OS-specific)
```rust
use crate::storage::StorageManager;

// Windows: Credential Manager
// macOS: Keychain
// Linux: Secret Service

async fn store_token(account_id: &str, token: &str) -> Result<()> {
    let storage = StorageManager::new()?;
    storage.set(account_id, token).await?;
    Ok(())
}

async fn retrieve_token(account_id: &str) -> Result<String> {
    let storage = StorageManager::new()?;
    storage.get(account_id).await
}
```

### File Operations
```rust
use tokio::fs;
use std::path::PathBuf;

async fn save_accounts(accounts: &[Account], path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(accounts)?;
    fs::write(path, json).await?;
    Ok(())
}

async fn load_accounts(path: &PathBuf) -> Result<Vec<Account>> {
    let content = fs::read_to_string(path).await?;
    let accounts: Vec<Account> = serde_json::from_str(&content)?;
    Ok(accounts)
}
```

### HTTP Requests
```rust
use reqwest;

async fn fetch_user_info(token: &str) -> Result<UserInfo> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.example.com/user")
        .bearer_auth(token)
        .send()
        .await?;
    
    let user_info: UserInfo = response.json().await?;
    Ok(user_info)
}
```

### WebSocket Server
```rust
use tokio_tungstenite::{accept_async, tungstenite::Message};

async fn handle_websocket(stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();
    
    while let Some(msg) = read.next().await {
        let msg = msg?;
        if msg.is_text() {
            // Handle message
            write.send(Message::Text("Response".to_string())).await?;
        }
    }
    
    Ok(())
}
```

## Type Definitions

### Serializable Types
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub token: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```

### Error Types
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Account not found: {0}")]
    NotFound(String),
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new("test@example.com", "Test User");
        assert_eq!(account.email, "test@example.com");
        assert_eq!(account.name, "Test User");
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = fetch_account("123").await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use trae_account_manager::account::AccountManager;

#[tokio::test]
async fn test_account_manager() {
    let mut manager = AccountManager::new();
    let account = manager.add_account("test", "token").await.unwrap();
    
    let retrieved = manager.get_account(&account.id).await.unwrap();
    assert_eq!(retrieved.name, "test");
}
```

## Project Structure

```
src-tauri/src/
├── account/            # Account management
│   ├── mod.rs         # Module exports
│   ├── account_manager.rs  # CRUD operations
│   ├── export_service.rs   # Export functionality
│   └── types.rs       # Account types
├── api/               # External API integration
│   ├── mod.rs
│   ├── trae_api.rs    # API client
│   └── types.rs       # API types
├── auth/              # Authentication
│   ├── mod.rs
│   └── token_manager.rs
├── storage/           # Secure storage
│   ├── mod.rs
│   ├── manager.rs     # Storage abstraction
│   ├── windows.rs     # Windows implementation
│   ├── macos.rs       # macOS implementation
│   └── linux.rs       # Linux implementation
├── commands/          # Tauri commands
│   ├── mod.rs
│   └── auth.rs
├── lib.rs            # Library root
└── main.rs           # Application entry point
```

## Dependencies

Common crates used in this project:

```toml
[dependencies]
tauri = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
reqwest = { version = "0.11", features = ["json"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
```

## Security Best Practices

- Never log sensitive data (tokens, passwords)
- Use OS-specific secure storage for credentials
- Validate all inputs from frontend
- Use HTTPS for all external API calls
- Implement rate limiting for API calls
- Clear sensitive data from memory after use
- Use Tauri's capability system for permission control
