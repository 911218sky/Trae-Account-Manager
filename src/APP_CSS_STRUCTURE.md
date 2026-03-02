# App.css Structure Guide

This document describes the organization of `App.css` for better maintainability.

## Table of Contents

The CSS file is organized into the following sections:

### 1. CSS Variables (Lines 1-100)
- Color palette (backgrounds, text, accents)
- Status colors (success, warning, danger, info)
- Borders and shadows
- Gradients
- Border radius values
- Spacing scale
- Transition timings

### 2. Base Styles (Lines 101-150)
- CSS reset
- Body and root element styles
- App container layout

### 3. Layout Components (Lines 151-400)
- **Sidebar** - Navigation sidebar with logo and menu items
- **App Content** - Main content wrapper
- **Page Header** - Common page header styles
- **App Header** - Top application header

### 4. Buttons & Controls (Lines 401-600)
- **Primary Buttons** - Add button, action buttons
- **Header Buttons** - Toolbar and header action buttons
- **Badge** - Notification badges
- **View Toggle** - Grid/List view switcher

### 5. Toolbar & Actions (Lines 601-800)
- **Toolbar** - Main toolbar container
- **Select All** - Bulk selection checkbox
- **Batch Actions** - Bulk operation buttons

### 6. Account Components (Lines 801-1500)
- **Account Grid** - Grid layout for account cards
- **Account Card** - Individual account card styles
  - Card header with avatar and info
  - Card status indicators
  - Card tags
  - Usage section with progress bars
  - Card meta information
  - Card footer

### 7. Context Menu (Lines 1501-1600)
- Context menu overlay
- Menu items and dividers
- Hover states

### 8. Modals (Lines 1601-2500)
- **Modal Base** - Common modal styles
- **Add Account Modal** - Account creation modal
- **Confirm Modal** - Confirmation dialogs
- **Detail Modal** - Account details view
- **Info Modal** - Information dialogs
- **Update Token Modal** - Token update form

### 9. Forms & Inputs (Lines 2501-3000)
- Input fields
- Textareas
- Select dropdowns
- Checkboxes and radio buttons
- Form validation states

### 10. Settings Page (Lines 3001-3500)
- Settings sections
- Machine ID card
- Path configuration
- Data management controls

### 11. Toast Notifications (Lines 3501-3700)
- Toast container
- Toast items with different types
- Toast animations

### 12. Switch Progress & Error Modals (Lines 3701-4100)
- **Switch Progress Modal** - Account switching progress
- **Switch Error Modal** - Error handling during switch

### 13. Loading States (Lines 4101-4300)
- Skeleton loaders
- Spinner animations
- Loading indicators

### 14. Animations (Lines 4301-4428)
- Keyframe animations
- Transition effects
- Hover animations

## Maintenance Guidelines

### Adding New Styles

1. **Find the appropriate section** - Use the table of contents above
2. **Add clear comments** - Describe what the styles are for
3. **Use CSS variables** - Reference existing variables for consistency
4. **Group related styles** - Keep component styles together

### Naming Conventions

- Use **kebab-case** for class names: `.account-card`, `.modal-overlay`
- Use **BEM-like** naming for variants: `.button-primary`, `.card-status-active`
- Prefix component-specific classes: `.account-*`, `.modal-*`, `.toolbar-*`

### Best Practices

1. **Use CSS Variables** - Always reference `:root` variables for colors, spacing, etc.
2. **Avoid Deep Nesting** - Keep specificity low (max 3 levels)
3. **Mobile First** - Add responsive styles with min-width media queries
4. **Comment Sections** - Use clear section headers with `/* === Section Name === */`
5. **Group by Component** - Keep all styles for a component together
6. **Consistent Spacing** - Use the spacing scale variables (`--space-*`)

### Example Section Format

```css
/* ============================================
   COMPONENT NAME
   ============================================ */

/* Component container */
.component-name {
  /* Layout */
  display: flex;
  /* Spacing */
  padding: var(--space-lg);
  /* Colors */
  background: var(--bg-primary);
  /* Effects */
  transition: var(--transition);
}

/* Component variants */
.component-name.variant {
  background: var(--accent);
}

/* Component children */
.component-name .child-element {
  color: var(--text-primary);
}
```

## Quick Reference

### Common Variables

```css
/* Colors */
--accent: #0ea5e9
--text-primary: #18181b
--bg-primary: #fafafa

/* Spacing */
--space-sm: 8px
--space-md: 12px
--space-lg: 16px

/* Shadows */
--shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.1)
--shadow-hover: 0 10px 40px rgba(14, 165, 233, 0.15)

/* Transitions */
--transition: 0.2s ease
--transition-fast: 0.15s ease
```

### Common Patterns

```css
/* Card hover effect */
.card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-hover);
}

/* Button with gradient */
.button-primary {
  background: var(--gradient-accent);
  box-shadow: var(--shadow-accent);
}

/* Glassmorphism effect */
.glass {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
}
```

## Refactoring Tips

If the file becomes too large (>5000 lines), consider:

1. Extract animations to a separate file
2. Move page-specific styles to component-level CSS modules
3. Use CSS-in-JS for highly dynamic styles
4. Create a design system with reusable utility classes

## Related Files

- `src/App.tsx` - Main application component
- `src/components/` - React components using these styles
- `AGENTS.md` - Development guidelines
