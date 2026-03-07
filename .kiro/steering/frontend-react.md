---
inclusion: fileMatch
fileMatchPattern: 'src/**/*.{tsx,ts,jsx,js}'
name: frontend-react
description: React and TypeScript development guidelines. Loaded when working with frontend code.
---

# Frontend Development Guidelines

## React & TypeScript Conventions

### Component Structure
- Use function components with Hooks (no class components)
- Props interface naming: `ComponentNameProps`
- Export components as default when single export
- Use named exports for utilities and types

```typescript
interface ButtonProps {
  label: string;
  onClick: () => void;
  disabled?: boolean;
}

export default function Button({ label, onClick, disabled }: ButtonProps) {
  return <button onClick={onClick} disabled={disabled}>{label}</button>;
}
```

### TypeScript Best Practices
- Use TypeScript strict mode
- Prefer `interface` over `type` for object shapes
- Use `const` for immutable values, `let` for mutable
- Prefer arrow functions for callbacks
- Use optional chaining (`?.`) and nullish coalescing (`??`)
- Avoid `any` - use `unknown` if type is truly unknown

### State Management
- Use React Context for global state (no Redux/Zustand)
- Use `useState` for local component state
- Use `useReducer` for complex state logic
- Custom hooks for reusable stateful logic

```typescript
// Custom hook example
export function useToast() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  
  const showToast = useCallback((message: string, type: ToastType) => {
    // Implementation
  }, []);
  
  return { toasts, showToast };
}
```

### Error Handling
Always use the logger service for errors:

```typescript
import { logger } from '@/services/logger';

try {
  await someOperation();
} catch (error) {
  logger.error('Operation failed', { error, context: 'ComponentName' });
  toast.error('Failed to complete operation');
}
```

### Performance Optimization
- Use `React.memo()` for expensive components
- Use `useMemo()` for expensive calculations
- Use `useCallback()` for callback functions passed to child components
- Implement virtualization for large lists (react-window)
- Lazy load heavy components with `React.lazy()`
- Debounce search inputs (300ms)

```typescript
const MemoizedList = React.memo(AccountList);

const filteredAccounts = useMemo(() => {
  return accounts.filter(acc => acc.name.includes(searchTerm));
}, [accounts, searchTerm]);

const handleSearch = useCallback(
  debounce((term: string) => setSearchTerm(term), 300),
  []
);
```

## Frontend-Backend Communication

### Calling Tauri Commands
```typescript
import { invoke } from '@tauri-apps/api/core';

// With parameters
const accounts = await invoke<Account[]>('get_accounts', {
  page: 1,
  pageSize: 50
});

// Error handling
try {
  await invoke('delete_account', { id: accountId });
} catch (error) {
  logger.error('Failed to delete account', { error, accountId });
}
```

### API Calls with Axios
```typescript
import api from '@/api';

// GET request
const response = await api.get<Account[]>('/accounts');

// POST request
const newAccount = await api.post<Account>('/accounts', {
  name: 'Account Name',
  token: 'token_value'
});

// Interceptor handles token refresh automatically
```

## Component Patterns

### Modal Pattern
```typescript
interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

export default function Modal({ isOpen, onClose, title, children }: ModalProps) {
  if (!isOpen) return null;
  
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <h2>{title}</h2>
        {children}
      </div>
    </div>
  );
}
```

### Form Handling
```typescript
function AddAccountForm() {
  const [formData, setFormData] = useState({ name: '', token: '' });
  
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    try {
      await invoke('add_account', formData);
      toast.success('Account added successfully');
    } catch (error) {
      logger.error('Failed to add account', { error, formData });
      toast.error('Failed to add account');
    }
  };
  
  return (
    <form onSubmit={handleSubmit}>
      {/* Form fields */}
    </form>
  );
}
```

## Key Services

### Logger Service
```typescript
import { logger } from '@/services/logger';

logger.debug('Debug message', { data });
logger.info('Info message', { data });
logger.warn('Warning message', { data });
logger.error('Error message', { error, context });
```

### WebSocket Client
```typescript
import { websocketClient } from '@/services/websocketClient';

// Connect
await websocketClient.connect();

// Listen for messages
websocketClient.on('account_updated', (account) => {
  // Handle update
});

// Send message
websocketClient.send('switch_account', { accountId });
```

### Account Cache
```typescript
import { accountCache } from '@/services/accountCache';

// Get cached accounts
const accounts = accountCache.getAccounts();

// Update cache
accountCache.updateAccount(updatedAccount);

// Clear cache
accountCache.clear();
```

## Testing

### Component Testing
```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import Button from './Button';

describe('Button', () => {
  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<Button label="Click me" onClick={handleClick} />);
    
    fireEvent.click(screen.getByText('Click me'));
    expect(handleClick).toHaveBeenCalledOnce();
  });
});
```

### Hook Testing
```typescript
import { renderHook, act } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { useToast } from './useToast';

describe('useToast', () => {
  it('adds toast on showToast', () => {
    const { result } = renderHook(() => useToast());
    
    act(() => {
      result.current.showToast('Test message', 'success');
    });
    
    expect(result.current.toasts).toHaveLength(1);
  });
});
```

## File Organization

```
src/
├── components/          # Reusable UI components
├── pages/              # Page-level components
├── hooks/              # Custom React hooks
├── contexts/           # React Context providers
├── services/           # Business logic and API clients
├── types/              # TypeScript type definitions
├── auth/               # Authentication logic
└── assets/             # Static assets (images, icons)
```

## Common Imports

```typescript
// React
import { useState, useEffect, useCallback, useMemo } from 'react';

// Tauri
import { invoke } from '@tauri-apps/api/core';

// Services
import { logger } from '@/services/logger';
import api from '@/api';

// Hooks
import { useToast } from '@/hooks/useToast';
import { useAuth } from '@/contexts/AuthContext';
```
