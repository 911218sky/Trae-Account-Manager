/**
 * Logger Service Exports
 * 
 * This file exports the singleton logger instance and factory function
 * for creating module-specific logger instances.
 * 
 * Configuration:
 * - Environment variables (VITE_LOG_LEVEL, VITE_LOG_FILE_ENABLED, etc.) are embedded at build time
 * - If no environment variables are set, defaults are used:
 *   - Log Level: 'info' (production) / 'debug' (development)
 *   - File Logging: Disabled
 *   - Pretty Print: Enabled in development only
 * 
 * Validates: Requirements 1.4, 4.1
 */

import { LoggerService } from './LoggerService';
import { ModuleLogger } from './types';

/**
 * Singleton Logger Service Instance
 * 
 * Initialized with environment-based configuration.
 * Use this for direct logging or to create module loggers.
 */
export const logger = new LoggerService();

/**
 * Factory function to create module-specific logger instances
 * 
 * @param moduleName - The name of the module
 * @returns A ModuleLogger instance for the specified module
 * 
 * Example:
 * ```typescript
 * const authLogger = createModuleLogger('auth');
 * authLogger.info('User logged in', { userId: '123' });
 * ```
 */
export const createModuleLogger = (moduleName: string): ModuleLogger => {
  return logger.createModuleLogger(moduleName);
};

// Re-export types for convenience
export type { LoggerConfig, LogContext, LogMessage, ModuleLogger, ILoggerService } from './types';
export { LoggerService } from './LoggerService';
