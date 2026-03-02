/**
 * Logger Service Type Definitions
 * 
 * This file contains all TypeScript interfaces and types for the unified logging system.
 * It defines the structure of logger configuration, log messages, and module logger instances.
 */

/**
 * LoggerConfig Interface
 * 
 * Configuration object for initializing the Logger Service.
 * Supports customization of log level, environment, file output, and pretty printing.
 * 
 * Validates: Requirements 1.1, 5.1, 7.1
 */
export interface LoggerConfig {
  /**
   * Log level threshold for output
   * Only messages at this level or higher severity will be output
   * @default 'info' in production, 'debug' in development
   */
  level?: 'debug' | 'info' | 'warn' | 'error';

  /**
   * Environment mode for logger configuration
   * Determines which transport and formatting to use
   * @default Detected from NODE_ENV environment variable
   */
  environment?: 'development' | 'production';

  /**
   * File output configuration
   * Enables writing logs to a file in addition to console output
   */
  fileOutput?: {
    /**
     * Whether to enable file output
     * @default false
     */
    enabled: boolean;

    /**
     * Path to the log file
     * Required if enabled is true
     */
    path: string;
  };

  /**
   * Whether to use pretty printing for console output
   * Only applicable in development environment
   * @default true in development, false in production
   */
  prettyPrint?: boolean;
}

/**
 * LogContext Interface
 * 
 * Context information included with log messages.
 * Contains the module name and any additional contextual data.
 * 
 * Validates: Requirements 1.1, 5.1, 7.1
 */
export interface LogContext {
  /**
   * Name of the module generating the log message
   * Used to identify the source of the log
   */
  module: string;

  /**
   * Additional contextual data
   * Can include any key-value pairs relevant to the log message
   */
  [key: string]: any;
}

/**
 * LogMessage Interface
 * 
 * Structure of a log message as it appears in output.
 * Contains all information about a single log entry.
 * 
 * Validates: Requirements 1.1, 5.1, 7.1
 */
export interface LogMessage {
  /**
   * Timestamp of the log message
   * Format: ISO 8601 in production, HH:MM:ss in development
   */
  timestamp: string;

  /**
   * Severity level of the log message
   */
  level: 'debug' | 'info' | 'warn' | 'error';

  /**
   * Name of the module that generated this log message
   */
  module: string;

  /**
   * The actual log message content
   */
  message: string;

  /**
   * Additional contextual information
   * Optional key-value pairs provided with the log message
   */
  context?: Record<string, any>;

  /**
   * Error stack trace (if applicable)
   * Included when logging error objects
   */
  stack?: string;

  /**
   * Process ID
   * Included in production environment logs
   */
  pid?: number;

  /**
   * Hostname
   * Included in production environment logs
   */
  hostname?: string;
}

/**
 * ModuleLogger Interface
 * 
 * Logger instance for a specific module.
 * Provides methods for logging at different levels with automatic module context.
 * 
 * Validates: Requirements 1.1, 5.1, 7.1
 */
export interface ModuleLogger {
  /**
   * Log a debug level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data to include with the message
   */
  debug(message: string, context?: Record<string, any>): void;

  /**
   * Log an info level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data to include with the message
   */
  info(message: string, context?: Record<string, any>): void;

  /**
   * Log a warn level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data to include with the message
   */
  warn(message: string, context?: Record<string, any>): void;

  /**
   * Log an error level message
   * 
   * @param message - The message to log
   * @param error - Optional error object or additional context
   */
  error(message: string, error?: Error | Record<string, any>): void;

  /**
   * Create a child logger with additional context
   * 
   * The child logger will include the provided context in all subsequent log messages
   * while maintaining the parent logger's module name and configuration.
   * 
   * @param context - Additional context to include in child logger
   * @returns A new ModuleLogger instance with the additional context
   */
  child(context: Record<string, any>): ModuleLogger;
}

/**
 * LoggerService Interface
 * 
 * Main logger service interface.
 * Provides methods for logging and managing the logger configuration.
 * 
 * Validates: Requirements 1.1, 5.1, 7.1
 */
export interface ILoggerService {
  /**
   * Log a debug level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data to include with the message
   */
  debug(message: string, context?: Record<string, any>): void;

  /**
   * Log an info level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data to include with the message
   */
  info(message: string, context?: Record<string, any>): void;

  /**
   * Log a warn level message
   * 
   * @param message - The message to log
   * @param context - Optional contextual data to include with the message
   */
  warn(message: string, context?: Record<string, any>): void;

  /**
   * Log an error level message
   * 
   * @param message - The message to log
   * @param error - Optional error object or additional context
   */
  error(message: string, error?: Error | Record<string, any>): void;

  /**
   * Set the log level threshold
   * 
   * Only messages at this level or higher severity will be output.
   * Changes take effect immediately.
   * 
   * @param level - The new log level
   */
  setLevel(level: 'debug' | 'info' | 'warn' | 'error'): void;

  /**
   * Get the current log level
   * 
   * @returns The current log level
   */
  getLevel(): string;

  /**
   * Create a module logger instance
   * 
   * Creates a new logger instance for a specific module.
   * All log messages from this instance will include the module name.
   * 
   * @param moduleName - The name of the module
   * @returns A ModuleLogger instance for the specified module
   */
  createModuleLogger(moduleName: string): ModuleLogger;
}
