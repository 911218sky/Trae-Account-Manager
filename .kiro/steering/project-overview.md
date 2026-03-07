---
inclusion: auto
name: project-overview
description: Quick project context including tech stack, architecture, conventions, and key directories. Use for general project understanding.
---

# Trae Account Manager - Project Overview

## Quick Context

This is a Tauri 2 desktop app for managing multiple accounts with React 19 frontend.

**Tech Stack**: React 19 + TypeScript + Vite | Tauri 2 + Rust

**Key Commands**:
- Dev: `npm run tauri dev`
- Build: `npm run tauri build`
- Test: `npm test`

## Architecture Overview

### Frontend (React)
```
src/
├── components/          # UI Components
│   ├── AccountList/    # Virtualized list with error boundary
│   ├── Modals/         # AddAccount, Detail, Confirm, Info, UpdateToken
│   └── UI Elements/    # Toast, ContextMenu, Sidebar, Progress
├── services/           # Business Logic Layer
│   ├── logger/         # Pino-based logging (LoggerService)
│   ├── accountCache.ts # Account data caching
│   ├── errorMonitor.ts # Error tracking and reporting
│   ├── performanceMonitor.ts # Performance metrics
│   └── websocketClient.ts # WebSocket connection
├── hooks/              # Custom React Hooks
│   ├── useToast.ts     # Toast notifications
│   ├── usePaginatedAccounts.ts # Account pagination
│   ├── useSelectionManager.ts # Multi-select logic
│   ├── useExportController.ts # Export functionality
│   └── usePerformanceMonitor.ts # Performance tracking
├── contexts/           # React Context
│   └── AuthContext.tsx # Authentication state
├── auth/               # Auth Logic
│   ├── interceptor.ts  # Axios interceptor for token refresh
│   └── requestQueue.ts # Queue requests during token refresh
├── pages/              # Page Components
│   ├── Dashboard.tsx   # Main account management page
│   └── Settings.tsx    # Settings page
└── types/              # TypeScript Types
    ├── index.ts        # Shared types
    └── pagination.ts   # Pagination types
```

### Backend (Rust/Tauri)
```
src-tauri/src/
├── account/            # Account Management
│   ├── account_manager.rs # CRUD operations
│   ├── export_service.rs  # Export to CSV/JSON
│   ├── file_manager.rs    # File operations
│   ├── stream_writer.rs   # Streaming export
│   └── types.rs           # Account types
├── api/                # External API Integration
│   ├── trae_api.rs     # API client
│   └── types.rs        # API types
├── auth/               # Authentication
│   └── token_manager.rs # Token storage and refresh
├── cli/                # CLI Commands
│   ├── commands.rs     # CLI command handlers
│   └── types.rs        # CLI types
├── commands/           # Tauri Commands
│   └── auth.rs         # Auth-related commands
├── hotkey/             # Global Hotkey
│   ├── manager.rs      # Hotkey registration
│   └── types.rs        # Hotkey types
├── storage/            # Secure Storage
│   ├── manager.rs      # Storage abstraction
│   ├── windows.rs      # Windows Credential Manager
│   ├── macos.rs        # macOS Keychain
│   ├── linux.rs        # Linux Secret Service
│   └── migration.rs    # Data migration
├── switcher/           # Account Switching
│   ├── core.rs         # Switch logic
│   ├── error.rs        # Error types
│   └── types.rs        # Switch types
├── tray/               # System Tray
│   └── manager.rs      # Tray menu and events
├── websocket/          # WebSocket Server
│   ├── server.rs       # WebSocket server
│   └── types.rs        # Message types
├── config.rs           # App configuration
├── login.rs            # Login window
├── machine.rs          # Machine ID generation
└── main.rs             # Entry point
```

## Important Conventions

### Code Style

#### TypeScript/React
- Use TypeScript strict mode
- Function components with Hooks (no class components)
- Props interface naming: `ComponentNameProps`
- Use `const` for immutable values, `let` for mutable
- Prefer arrow functions for callbacks
- Use optional chaining (`?.`) and nullish coalescing (`??`)
- Error handling through unified logger service

#### Rust
- Follow Rust standard naming conventions (snake_case for functions/variables)
- Use `Result<T, E>` for error handling
- Prefer `&str` over `String` for function parameters
- Use `derive` macros for common traits
- Document public APIs with `///` comments
- Use `#[tauri::command]` for frontend-callable functions

### CSS Organization
- **Single file**: All styles in `src/App.css` only
- Use CSS variables from `:root` for colors, spacing, transitions
- Structure documented in `src/APP_CSS_STRUCTURE.md`
- Never create separate CSS files per component
- Use clear section comments: `/* === Section Name === */`
- Keep component-related styles grouped together
- Use kebab-case for class names: `.account-card`, `.modal-overlay`
- Mobile-first responsive design with media queries

### Git Commits (English only)
Format: `<type>(<scope>): <subject>`

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semicolons, etc.)
- `refactor`: Code refactoring without changing functionality
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Build process or auxiliary tool changes
- `ci`: CI/CD configuration changes

**Examples**:
```bash
feat(auth): add token refresh mechanism
fix(ui): resolve account list rendering issue
docs(readme): update installation instructions
refactor(api): simplify error handling logic
chore(deps): update tauri to version 2.1.0
perf(list): implement virtual scrolling for large datasets
```

### Environment Variables
- Prefix: `VITE_` (embedded at build time by Vite)
- Dev: `.env.development` (auto-loaded in dev mode)
- Prod: `.env.production` (auto-loaded in build)
- App has hardcoded defaults if env files missing
- Available variables:
  - `VITE_NODE_ENV`: `production` or `development`
  - `VITE_LOG_LEVEL`: `debug`, `info`, `warn`, `error`
  - `VITE_LOG_FILE_ENABLED`: `true` or `false`
  - `VITE_LOG_FILE_PATH`: Log file path

### Testing Strategy
- Unit tests use Vitest
- Test files placed next to their corresponding modules
- Test naming: `*.test.ts` or `*.test.tsx`
- Mock external dependencies (API calls, Tauri commands)
- Focus on business logic and edge cases
- Run tests before committing: `npm test`

## Common Patterns

### Frontend-Backend Communication
```typescript
// Frontend: Call Tauri command
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<Account[]>('get_accounts', {
  page: 1,
  pageSize: 50
});
```

```rust
// Backend: Tauri command
#[tauri::command]
async fn get_accounts(
    page: usize,
    page_size: usize,
    state: State<'_, AppState>
) -> Result<Vec<Account>, String> {
    // Implementation
}
```

### Error Handling
```typescript
// Frontend: Use logger service
import { logger } from '@/services/logger';

try {
  await someOperation();
} catch (error) {
  logger.error('Operation failed', { error, context: 'ComponentName' });
  toast.error('Failed to complete operation');
}
```

```rust
// Backend: Return Result
use anyhow::{Context, Result};

fn some_operation() -> Result<Data> {
    let data = fetch_data()
        .context("Failed to fetch data")?;
    Ok(data)
}
```

### State Management
```typescript
// Use React Context for global state
import { useAuth } from '@/contexts/AuthContext';

function Component() {
  const { user, login, logout } = useAuth();
  // Use auth state
}
```

## Performance Considerations

- Use virtualized lists for large datasets (AccountList uses react-window)
- Implement pagination for API calls (50 items per page)
- Cache frequently accessed data (accountCache service)
- Debounce search inputs (300ms)
- Lazy load modals and heavy components
- Monitor performance with usePerformanceMonitor hook
- Use WebSocket for real-time updates instead of polling

## Security Best Practices

- Store tokens in OS-specific secure storage (Credential Manager/Keychain)
- Never log sensitive data (tokens, passwords)
- Validate all user inputs on both frontend and backend
- Use HTTPS for all API calls
- Implement token refresh mechanism with request queuing
- Clear sensitive data on logout
- Use Tauri's capability system for permission control

## When You Need More Details

Reference specific documentation:
- **Full guidelines**: `#[[file:AGENTS.md]]`
- **CSS structure**: `#[[file:src/APP_CSS_STRUCTURE.md]]`
- **Project README**: `#[[file:README.md]]`
- **Logger implementation**: Check `src/services/logger/` directory
- **API types**: Check `src/types/index.ts`
- **Tauri config**: Check `src-tauri/tauri.conf.json`

## Quick Reference

**Frontend State**: React Context (no Redux/Zustand)
**Logging**: Pino-based logger service
**Charts**: Recharts library
**Testing**: Vitest
**HTTP Client**: Axios with interceptors
**Icons**: React Icons
**Styling**: Pure CSS with variables
**Build Tool**: Vite
**Package Manager**: npm

---

💡 This overview covers most common scenarios. Reference specific files only when you need implementation details.
