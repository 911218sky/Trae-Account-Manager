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

### Default Environment (.env)

The default `.env` file uses **production** configuration:

```ini
NODE_ENV=production
LOG_LEVEL=info
LOG_FILE_ENABLED=true
LOG_FILE_PATH=./logs/app.log
```

### Development Environment (.env.development)

For development, create `.env.development`:

```ini
NODE_ENV=development
LOG_LEVEL=debug
LOG_FILE_ENABLED=false
```

### Environment Variables

- `NODE_ENV`: Environment mode (`production` or `development`)
- `LOG_LEVEL`: Logging level (`debug`, `info`, `warn`, `error`)
- `LOG_FILE_ENABLED`: Enable file logging (`true` or `false`)
- `LOG_FILE_PATH`: Log file path

## Related Documentation

- [README.md](./README.md) - Project overview
- [src/services/logger/](./src/services/logger/) - Logger implementation docs
