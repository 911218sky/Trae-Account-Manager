---
inclusion: fileMatch
fileMatchPattern: 'src/App.css'
name: css-styling
description: CSS styling guidelines and conventions. Loaded when working with App.css.
---

# CSS Styling Guidelines

## Critical Rules

- **Single CSS file**: ALL styles must be in `src/App.css` only
- **Never create separate CSS files** per component
- Use CSS variables from `:root` for consistency
- Follow the structure documented in `src/APP_CSS_STRUCTURE.md`

## CSS Variables

Use existing CSS variables for consistency:

```css
:root {
  /* Colors */
  --primary-color: #007bff;
  --secondary-color: #6c757d;
  --success-color: #28a745;
  --danger-color: #dc3545;
  --warning-color: #ffc107;
  --info-color: #17a2b8;
  
  /* Background */
  --bg-primary: #ffffff;
  --bg-secondary: #f8f9fa;
  --bg-dark: #343a40;
  
  /* Text */
  --text-primary: #212529;
  --text-secondary: #6c757d;
  --text-muted: #adb5bd;
  
  /* Spacing */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  --spacing-xl: 32px;
  
  /* Border */
  --border-radius: 4px;
  --border-color: #dee2e6;
  
  /* Transitions */
  --transition-fast: 150ms;
  --transition-normal: 300ms;
  --transition-slow: 500ms;
}
```

## Naming Conventions

- Use **kebab-case** for class names
- Component-based naming: `.component-name`
- State modifiers: `.component-name--state`
- Child elements: `.component-name__element`

```css
/* Good */
.account-card { }
.account-card--selected { }
.account-card__title { }
.modal-overlay { }
.button--primary { }

/* Bad */
.AccountCard { }
.account_card { }
.accountCard { }
```

## File Organization

Use clear section comments to organize styles:

```css
/* === Reset & Base === */
* { box-sizing: border-box; }

/* === Layout === */
.container { }
.sidebar { }

/* === Components === */
/* Account Card */
.account-card { }

/* Modal */
.modal-overlay { }

/* === Utilities === */
.text-center { }
.mt-4 { }
```

## Component Styling Pattern

Group all related styles together:

```css
/* === Account Card === */
.account-card {
  padding: var(--spacing-md);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-primary);
  transition: all var(--transition-normal);
}

.account-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.account-card--selected {
  border-color: var(--primary-color);
  background: rgba(0, 123, 255, 0.05);
}

.account-card__title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: var(--spacing-sm);
}

.account-card__email {
  font-size: 14px;
  color: var(--text-secondary);
}
```

## Responsive Design

Mobile-first approach with media queries:

```css
/* Mobile first (default) */
.container {
  padding: var(--spacing-sm);
}

/* Tablet */
@media (min-width: 768px) {
  .container {
    padding: var(--spacing-md);
  }
}

/* Desktop */
@media (min-width: 1024px) {
  .container {
    padding: var(--spacing-lg);
    max-width: 1200px;
    margin: 0 auto;
  }
}
```

## Common Patterns

### Flexbox Layout
```css
.flex-container {
  display: flex;
  gap: var(--spacing-md);
  align-items: center;
  justify-content: space-between;
}

.flex-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}
```

### Grid Layout
```css
.grid-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: var(--spacing-md);
}
```

### Modal Overlay
```css
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: var(--spacing-lg);
  max-width: 500px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
}
```

### Button Styles
```css
.button {
  padding: var(--spacing-sm) var(--spacing-md);
  border: none;
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.button--primary {
  background: var(--primary-color);
  color: white;
}

.button--primary:hover {
  background: #0056b3;
}

.button--secondary {
  background: var(--secondary-color);
  color: white;
}

.button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

### Form Elements
```css
.form-group {
  margin-bottom: var(--spacing-md);
}

.form-label {
  display: block;
  margin-bottom: var(--spacing-xs);
  font-weight: 500;
  color: var(--text-primary);
}

.form-input {
  width: 100%;
  padding: var(--spacing-sm);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  transition: border-color var(--transition-fast);
}

.form-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1);
}

.form-input--error {
  border-color: var(--danger-color);
}
```

### Toast Notifications
```css
.toast-container {
  position: fixed;
  top: var(--spacing-lg);
  right: var(--spacing-lg);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.toast {
  padding: var(--spacing-md);
  border-radius: var(--border-radius);
  background: white;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 300px;
  animation: slideIn var(--transition-normal);
}

.toast--success {
  border-left: 4px solid var(--success-color);
}

.toast--error {
  border-left: 4px solid var(--danger-color);
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
```

## Utility Classes

Create reusable utility classes:

```css
/* Text alignment */
.text-left { text-align: left; }
.text-center { text-align: center; }
.text-right { text-align: right; }

/* Spacing */
.mt-1 { margin-top: var(--spacing-xs); }
.mt-2 { margin-top: var(--spacing-sm); }
.mt-3 { margin-top: var(--spacing-md); }
.mt-4 { margin-top: var(--spacing-lg); }

.mb-1 { margin-bottom: var(--spacing-xs); }
.mb-2 { margin-bottom: var(--spacing-sm); }
.mb-3 { margin-bottom: var(--spacing-md); }
.mb-4 { margin-bottom: var(--spacing-lg); }

/* Display */
.d-none { display: none; }
.d-block { display: block; }
.d-flex { display: flex; }
.d-grid { display: grid; }

/* Visibility */
.hidden { visibility: hidden; }
.visible { visibility: visible; }
```

## Performance Tips

- Avoid deep nesting (max 3 levels)
- Use CSS transforms for animations (better performance)
- Minimize use of expensive properties (box-shadow, filter)
- Use `will-change` sparingly for animations

```css
/* Good: Use transform for animations */
.card {
  transition: transform var(--transition-normal);
}

.card:hover {
  transform: translateY(-2px);
}

/* Use will-change for complex animations */
.animated-element {
  will-change: transform, opacity;
}
```

## Dark Mode Support (Future)

Prepare for dark mode with CSS variables:

```css
:root {
  --bg-primary: #ffffff;
  --text-primary: #212529;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #1a1a1a;
    --text-primary: #f8f9fa;
  }
}
```

## Reference

For complete CSS structure, see: `#[[file:src/APP_CSS_STRUCTURE.md]]`
