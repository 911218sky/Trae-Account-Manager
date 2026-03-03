# AI Development Guidelines

This project uses AI-assisted development. Below are the configurations and guidelines.

## Project Structure

```
Trae-Account-Manager/
├── src/                    # React frontend code
│   ├── components/        # React components
│   ├── services/          # Service layer (API, cache, logger, etc.)
│   ├── hooks/             # React Hooks
│   ├── contexts/          # React Context
│   └── pages/             # Page components
├── src-tauri/             # Tauri backend code (Rust)
└── .github/workflows/     # GitHub Actions workflows
```

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Tauri 2 + Rust
- **State Management**: React Context
- **Logging**: Pino
- **Charts**: Recharts

## Development Commands

```bash
# Install dependencies
npm install

# Development mode
npm run tauri dev

# Build application
npm run tauri build

# Run tests
npm test
```

## Release Process

1. Update version numbers in `package.json` and `src-tauri/Cargo.toml`
2. Create and push version tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. GitHub Actions will automatically build and create a Release

## AI Development Guidelines

### Files to Exclude from Git

- `.kiro/` - Kiro AI configuration and cache
- `.env` - Environment variables (use .env.production as default)
- `.vscode/` - VS Code editor settings
- `node_modules/` - Dependencies
- `dist/` - Build output
- `src-tauri/target/` - Rust build output

### Git Commit Guidelines

All commit messages MUST be in English and follow the Conventional Commits specification:

**Format**: `<type>(<scope>): <subject>`

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
```

### Code Style

- Use TypeScript strict mode
- React components use function components and Hooks
- Error handling uses unified logging system
- Follow Rust standard naming conventions

### CSS Organization

- All styles are in `src/App.css` - do not create separate CSS files per component
- Use CSS variables from `:root` for consistency
- Follow the structure documented in `src/APP_CSS_STRUCTURE.md`
- Use clear section comments: `/* === Section Name === */`
- Keep component-related styles grouped together
- Use kebab-case for class names: `.account-card`, `.modal-overlay`

### Testing

- Unit tests use Vitest
- Test files should be placed next to their corresponding modules

## Environment Configuration

### How Environment Variables Work

Vite embeds environment variables at **build time**. Variables prefixed with `VITE_` are accessible via `import.meta.env` in the application code.

- **Development**: Vite loads `.env.development` automatically when running `npm run tauri dev`
- **Production**: Vite loads `.env.production` automatically when running `npm run tauri build`
- **Runtime**: The built app uses values embedded during build - no `.env` files are included in the final package

### Default Configuration (Hardcoded in App)

When environment variables are not set, the app uses these defaults:

- **Log Level**: `info` (production) / `debug` (development)
- **File Logging**: Disabled by default
- **Pretty Print**: Enabled in development, disabled in production

These defaults are defined in `src/services/logger/LoggerService.ts` and will be used if no `.env` files are present during build.

### Production Environment (.env.production)

Optional configuration for production builds:

```ini
VITE_NODE_ENV=production
VITE_LOG_LEVEL=info
VITE_LOG_FILE_ENABLED=true
VITE_LOG_FILE_PATH=./logs/app.log
```

### Development Environment (.env.development)

Optional configuration for development mode:

```ini
VITE_NODE_ENV=development
VITE_LOG_LEVEL=debug
VITE_LOG_FILE_ENABLED=false
```

### Environment Variables

All environment variables must use the `VITE_` prefix to be accessible in the app:

- `VITE_NODE_ENV`: Environment mode (`production` or `development`)
- `VITE_LOG_LEVEL`: Logging level (`debug`, `info`, `warn`, `error`)
- `VITE_LOG_FILE_ENABLED`: Enable file logging (`true` or `false`)
- `VITE_LOG_FILE_PATH`: Log file path (only used when file logging is enabled)

**Note**: If `.env.production` is missing during build, the app will use hardcoded defaults. The built application does not require any `.env` files to run.

## Related Documentation

- [README.md](./README.md) - Project overview
- [src/services/logger/](./src/services/logger/) - Logger implementation docs
